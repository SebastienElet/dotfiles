use super::super::MeasureError;
use super::super::hook::now_ms;
use super::super::store::{open_private_append, open_private_new, temporary_path};
use super::io::read_optional_json;
use super::records::read_events_file;
use super::{FinishArgs, MergeReady, ResultRecord, open_run, open_store, validate_result_record};
use serde::Serialize;
use serde_json::json;
use std::fs;
use std::fs::File;
use std::io::{Seek, SeekFrom, Write};
use std::path::Path;

#[cfg(test)]
mod tests;

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
    let events_path = run_dir.join("events.jsonl");
    let mut events = open_private_append(&events_path)?;
    events.lock()?;
    read_events_file(&mut events, &args.run_id)?;
    let revision = current
        .map_or(Ok(1), |result| result.revision.checked_add(1).ok_or(()))
        .map_err(|()| MeasureError::new("result revision overflow"))?;
    let result = build(args, revision);
    let result_path = run_dir.join("result.json");
    let temporary = prepare_result(&result_path, &result)?;
    let event = result_event_bytes(&result)?;
    let committed = append_then_commit(&mut events, &event, || {
        fs::rename(&temporary, &result_path).map_err(Into::into)
    });
    if committed.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    committed
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

fn result_event_bytes(result: &ResultRecord) -> Result<Vec<u8>, MeasureError> {
    let event = ResultEvent {
        timestamp_ms: result.recorded_at_ms,
        event_id: format!("result-{}", result.revision),
        event: "result_recorded",
        native_event: "human.finish",
        artifact: "result.json",
        native_ids: json!({"revision": result.revision}),
        result,
    };
    let mut bytes = serde_json::to_vec(&event)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn prepare_result(path: &Path, result: &ResultRecord) -> Result<std::path::PathBuf, MeasureError> {
    let temporary = temporary_path(path);
    let mut file = open_private_new(&temporary)?;
    file.write_all(&serde_json::to_vec_pretty(result)?)?;
    file.sync_all()?;
    Ok(temporary)
}

fn append_then_commit(
    events: &mut File,
    event: &[u8],
    commit: impl FnOnce() -> Result<(), MeasureError>,
) -> Result<(), MeasureError> {
    let previous = events.metadata()?.len();
    events.seek(SeekFrom::End(0))?;
    let outcome = events
        .write_all(event)
        .and_then(|()| events.sync_data())
        .map_err(MeasureError::from)
        .and_then(|()| commit());
    if let Err(error) = outcome {
        rollback_event(events, previous)?;
        return Err(error);
    }
    Ok(())
}

fn rollback_event(events: &mut File, previous: u64) -> Result<(), MeasureError> {
    events.set_len(previous)?;
    events.seek(SeekFrom::End(0))?;
    events.sync_data()?;
    Ok(())
}
