use super::super::MeasureError;
use super::super::model::{PromptRecord, RunRecord};
use super::super::store::ManagedPath;
use super::MergeReady;
use super::io::visit_jsonl_typed;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

mod history;
pub use history::{
    EventHistory, ResultState, latest_result, read_events_file, read_events_for_list_with,
    read_events_with, result_state,
};

#[derive(Clone, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResultRecord {
    pub schema_version: u8,
    pub run_id: String,
    pub revision: u64,
    pub recorded_at_ms: u64,
    pub merge_ready: MergeReady,
    pub human_minutes: f64,
    pub human_edited_diff: bool,
    pub failure_reason: Option<String>,
    pub evidence: Vec<String>,
    pub regression: bool,
    pub invariants: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StoredEvent {
    pub timestamp_ms: u64,
    pub event_id: String,
    pub event: String,
    pub native_event: String,
    pub artifact: String,
    pub native_ids: Map<String, Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<ResultRecord>,
}

pub fn read_first_prompt(
    path: &ManagedPath,
    run: &RunRecord,
) -> Result<Option<PromptRecord>, MeasureError> {
    let mut first = None;
    visit_jsonl_typed::<PromptRecord>(path, "prompts.jsonl", |prompt| {
        validate_prompt(&prompt, run)?;
        if first.is_none() {
            first = Some(prompt);
        }
        Ok(())
    })?;
    Ok(first)
}

pub fn validate_result_record(result: &ResultRecord, run_id: &str) -> Result<(), MeasureError> {
    if result.schema_version != 1 || result.run_id != run_id {
        return Err(MeasureError::new(
            "managed result.json has an unexpected identity",
        ));
    }
    if result.revision == 0 {
        return Err(MeasureError::new(
            "managed result.json has an invalid revision",
        ));
    }
    if result.recorded_at_ms == 0
        || !result.human_minutes.is_finite()
        || result.human_minutes < 0.0
        || has_empty(&result.evidence)
        || has_empty(&result.invariants)
    {
        return Err(MeasureError::new(
            "managed result.json has invalid measurement values",
        ));
    }
    validate_verdict(result)
}

fn validate_prompt(prompt: &PromptRecord, run: &RunRecord) -> Result<(), MeasureError> {
    if prompt.timestamp_ms == 0
        || prompt.session_id != run.session_id
        || !is_digest(&prompt.event_id)
    {
        return Err(MeasureError::new(
            "managed prompts.jsonl has an invalid record",
        ));
    }
    Ok(())
}

fn invalid_event(event: &StoredEvent, run_id: &str) -> bool {
    if event.timestamp_ms == 0
        || event.event.is_empty()
        || event.native_event.is_empty()
        || event.artifact.is_empty()
        || event
            .native_ids
            .values()
            .any(|value| !value.is_string() && !value.is_number())
    {
        return true;
    }
    match &event.result {
        Some(result) => {
            event.event != "result_recorded"
                || event.native_event != "human.finish"
                || event.artifact != "result.json"
                || event.event_id != format!("result-{}", result.revision)
                || event.timestamp_ms != result.recorded_at_ms
                || event.native_ids.len() != 1
                || event.native_ids.get("revision").and_then(Value::as_u64) != Some(result.revision)
                || validate_result_record(result, run_id).is_err()
        }
        None => event.event == "result_recorded" || !is_digest(&event.event_id),
    }
}

fn validate_verdict(result: &ResultRecord) -> Result<(), MeasureError> {
    let reason = result
        .failure_reason
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    match result.merge_ready {
        MergeReady::Fail if !reason => Err(MeasureError::new(
            "managed result.json requires a failure reason",
        )),
        MergeReady::Pass if result.failure_reason.is_some() => Err(MeasureError::new(
            "managed result.json forbids a failure reason for pass",
        )),
        MergeReady::Unjudgeable if result.evidence.is_empty() => Err(MeasureError::new(
            "managed result.json requires evidence for unjudgeable",
        )),
        _ => Ok(()),
    }
}

pub(super) fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn has_empty(values: &[String]) -> bool {
    values.iter().any(|value| value.trim().is_empty())
}
