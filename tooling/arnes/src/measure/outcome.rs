use super::MeasureError;
use super::hook::now_ms;
use super::result::{open_run, open_store, visit_jsonl_typed};
use super::store::{append_jsonl_bytes, jsonl_bytes};
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args)]
#[command(
    about = "Record an explicit outcome backed by a named oracle",
    long_about = "Record an explicit outcome backed by a named oracle; this does not infer success from agent activity or termination"
)]
pub struct OutcomeArgs {
    pub run_id: String,
    #[arg(long, value_enum, help = "Set pass, fail, or unjudgeable explicitly")]
    pub status: OutcomeStatus,
    #[arg(long, help = "Name the oracle supporting a pass or fail")]
    pub oracle: Option<String>,
    #[arg(long, value_enum, help = "Classify why the run is unjudgeable")]
    pub reason: Option<UnjudgeableReason>,
    #[arg(long, help = "Append a replacement revision when the outcome differs")]
    pub replace: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum OutcomeStatus {
    Pass,
    Fail,
    Unjudgeable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum UnjudgeableReason {
    MissingOracle,
    IncompleteRun,
    MeasurementFailure,
    NotComparable,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OutcomeRecord {
    schema_version: u8,
    run_id: String,
    revision: u64,
    recorded_at_ms: u64,
    status: OutcomeStatus,
    oracle: Option<String>,
    reason: Option<UnjudgeableReason>,
}

impl OutcomeRecord {
    pub fn status(&self) -> OutcomeStatus {
        self.status
    }

    pub fn recorded_at_ms(&self) -> u64 {
        self.recorded_at_ms
    }
}

pub fn record(args: OutcomeArgs) -> Result<(), MeasureError> {
    validate_input(&args)?;
    let store = open_store()?;
    let run = open_run(&store, &args.run_id)?;
    if super::store::validation::read_run(&run.join("run.json"))?.schema_version() != 2 {
        return Err(MeasureError::new(
            "measure outcome supports only v2 runs; use measure finish",
        ));
    }
    let lock = store.open_run_lock(&args.run_id)?;
    lock.lock()?;
    let path = run.join("outcomes.jsonl");
    let latest = latest(&path, &args.run_id)?;
    if latest
        .as_ref()
        .is_some_and(|record| equivalent(record, &args))
    {
        return Ok(());
    }
    if latest.is_some() && !args.replace {
        return Err(MeasureError::new("outcome already differs; use --replace"));
    }
    let recorded_at_ms = now_ms();
    if latest
        .as_ref()
        .is_some_and(|record| recorded_at_ms < record.recorded_at_ms)
    {
        return Err(MeasureError::new("outcome timestamps must be monotonic"));
    }
    let revision = latest
        .map_or(Ok(1), |record| record.revision.checked_add(1).ok_or(()))
        .map_err(|()| MeasureError::new("outcome revision overflow"))?;
    let record = OutcomeRecord {
        schema_version: 1,
        run_id: args.run_id,
        revision,
        recorded_at_ms,
        status: args.status,
        oracle: args.oracle,
        reason: args.reason,
    };
    append_jsonl_bytes(&path, &jsonl_bytes(&record)?)
}

fn validate_input(args: &OutcomeArgs) -> Result<(), MeasureError> {
    match args.status {
        OutcomeStatus::Pass | OutcomeStatus::Fail => {
            let oracle = args.oracle.as_deref().ok_or_else(|| {
                MeasureError::new("oracle is required for pass and fail outcomes")
            })?;
            if args.reason.is_some() {
                return Err(MeasureError::new(
                    "reason is forbidden for pass and fail outcomes",
                ));
            }
            if !valid_oracle(oracle) {
                return Err(MeasureError::new(
                    "oracle must be a lowercase ASCII identifier",
                ));
            }
        }
        OutcomeStatus::Unjudgeable => {
            if args.oracle.is_some() {
                return Err(MeasureError::new(
                    "oracle is forbidden for unjudgeable outcomes",
                ));
            }
            if args.reason.is_none() {
                return Err(MeasureError::new(
                    "reason is required for unjudgeable outcomes",
                ));
            }
        }
    }
    Ok(())
}

fn valid_oracle(value: &str) -> bool {
    (1..=64).contains(&value.len())
        && value.bytes().enumerate().all(|(index, byte)| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || (index > 0 && matches!(byte, b'-' | b'_' | b'.'))
        })
}

pub(super) fn latest(
    path: &super::store::ManagedPath,
    run_id: &str,
) -> Result<Option<OutcomeRecord>, MeasureError> {
    let mut latest: Option<OutcomeRecord> = None;
    visit_jsonl_typed(path, "outcomes.jsonl", |record: OutcomeRecord| {
        let expected_revision = latest.as_ref().map_or(Ok(1), |value| {
            value
                .revision
                .checked_add(1)
                .ok_or_else(|| MeasureError::new("outcome revision overflow"))
        })?;
        let previous_timestamp = latest.as_ref().map(|value| value.recorded_at_ms);
        validate_record(&record, run_id, expected_revision, previous_timestamp)?;
        latest = Some(record);
        Ok(())
    })?;
    Ok(latest)
}

fn validate_record(
    record: &OutcomeRecord,
    run_id: &str,
    expected_revision: u64,
    previous_timestamp: Option<u64>,
) -> Result<(), MeasureError> {
    if previous_timestamp.is_some_and(|timestamp| record.recorded_at_ms < timestamp) {
        return Err(MeasureError::new("outcome timestamps must be monotonic"));
    }
    let valid_shape = record.schema_version == 1
        && record.run_id == run_id
        && record.revision == expected_revision
        && record.recorded_at_ms > 0;
    let valid_fields = match record.status {
        OutcomeStatus::Pass | OutcomeStatus::Fail => {
            record.oracle.as_deref().is_some_and(valid_oracle) && record.reason.is_none()
        }
        OutcomeStatus::Unjudgeable => record.oracle.is_none() && record.reason.is_some(),
    };
    if valid_shape && valid_fields {
        Ok(())
    } else {
        Err(MeasureError::new(
            "managed outcomes.jsonl has an invalid record",
        ))
    }
}

fn equivalent(record: &OutcomeRecord, args: &OutcomeArgs) -> bool {
    record.status == args.status && record.oracle == args.oracle && record.reason == args.reason
}
