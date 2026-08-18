use super::super::MeasureError;
use super::super::hook::now_ms;
use super::super::store::{append_jsonl, open_private_append, write_json_atomic};
use super::io::{read_jsonl, read_optional_json};
use super::{FinishArgs, MergeReady, ResultRecord, open_run, open_store, validate_result_record};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct ResultEvent<'a> {
    timestamp_ms: u64,
    event_id: String,
    event: &'static str,
    native_event: &'static str,
    artifact: &'static str,
    native_ids: serde_json::Value,
    result: &'a ResultRecord,
}

pub fn record(args: FinishArgs) -> Result<(), MeasureError> {
    validate(&args)?;
    let store = open_store()?;
    let run_dir = open_run(&store, &args.run_id)?;
    let lock = open_private_append(&run_dir.join("result.lock"))?;
    lock.lock()?;
    let current: Option<ResultRecord> =
        read_optional_json(&run_dir.join("result.json"), "result.json")?;
    if let Some(result) = &current {
        validate_result_record(result, &args.run_id)?;
    }
    read_jsonl(&run_dir.join("events.jsonl"), "events.jsonl")?;
    let revision = current
        .map_or(Ok(1), |result| result.revision.checked_add(1).ok_or(()))
        .map_err(|()| MeasureError::new("result revision overflow"))?;
    let result = build(args, revision);
    write_json_atomic(&run_dir.join("result.json"), &result)?;
    append_result_event(&run_dir, &result)
}

fn validate(args: &FinishArgs) -> Result<(), MeasureError> {
    if !args.human_minutes.is_finite() || args.human_minutes < 0.0 {
        return Err(MeasureError::new(
            "human minutes must be finite non-negative",
        ));
    }
    validate_texts(&args.evidence, "evidence")?;
    validate_texts(&args.invariant, "invariant")?;
    match args.merge_ready {
        MergeReady::Fail if !present(args.failure_reason.as_deref()) => Err(MeasureError::new(
            "failure reason is required when merge-ready is fail",
        )),
        MergeReady::Pass if args.failure_reason.is_some() => Err(MeasureError::new(
            "failure reason is forbidden when merge-ready is pass",
        )),
        MergeReady::Unjudgeable if args.evidence.is_empty() => Err(MeasureError::new(
            "evidence is required when merge-ready is unjudgeable",
        )),
        _ => Ok(()),
    }
}

fn validate_texts(values: &[String], label: &str) -> Result<(), MeasureError> {
    if values.iter().all(|value| present(Some(value))) {
        Ok(())
    } else {
        Err(MeasureError::new(format!("{label} cannot be empty")))
    }
}

fn present(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn build(args: FinishArgs, revision: u64) -> ResultRecord {
    ResultRecord {
        schema_version: 1,
        run_id: args.run_id,
        revision,
        recorded_at_ms: now_ms(),
        merge_ready: args.merge_ready,
        human_minutes: args.human_minutes,
        human_edited_diff: args.human_edited_diff,
        failure_reason: args.failure_reason,
        evidence: args.evidence,
        regression: args.regression,
        invariants: args.invariant,
    }
}

fn append_result_event(
    run_dir: &std::path::Path,
    result: &ResultRecord,
) -> Result<(), MeasureError> {
    let event = ResultEvent {
        timestamp_ms: result.recorded_at_ms,
        event_id: format!("result-{}", result.revision),
        event: "result_recorded",
        native_event: "human.finish",
        artifact: "result.json",
        native_ids: json!({"revision": result.revision}),
        result,
    };
    append_jsonl(&run_dir.join("events.jsonl"), &event)
}
