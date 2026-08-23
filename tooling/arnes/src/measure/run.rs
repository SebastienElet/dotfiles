use super::MeasureError;
use super::fingerprint;
use super::model::{HookAgent, RunRecord};
use super::redaction::redact_string;
use super::repository;
use serde_json::Value;
use std::env;
use std::path::{Path, PathBuf};

pub struct NewRun<'a> {
    pub agent: HookAgent,
    pub session: &'a str,
    pub run_id: String,
    pub timestamp_ms: u64,
    pub raw: &'a Value,
    pub repository_root: Option<String>,
    pub observed: &'a Path,
    pub deployment_root: &'a Path,
}

pub fn build(input: NewRun<'_>) -> Result<RunRecord, MeasureError> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| MeasureError::new("HOME is required for harness fingerprinting"))?;
    let fingerprint = fingerprint::deployed(input.agent, &home, input.deployment_root)?;
    let repository = input
        .repository_root
        .map(|root| repository::observe(input.observed, root));
    Ok(RunRecord {
        schema_version: 1,
        run_id: input.run_id,
        agent: input.agent.as_str().to_owned(),
        session_id: input.session.to_owned(),
        started_at_ms: input.timestamp_ms,
        model: model(input.raw).map(|model| redact_string(&model)),
        repository: repository.as_ref().map(|value| value.root.clone()),
        repository_commit: repository.as_ref().and_then(|value| value.head.clone()),
        repository_branch: repository.as_ref().and_then(|value| value.branch.clone()),
        repository_dirty: repository.map(|value| value.dirty),
        harness_fingerprint: fingerprint.digest,
        harness_fingerprint_limitations: fingerprint.limitations,
    })
}

fn model(value: &Value) -> Option<String> {
    value
        .get("model")
        .or_else(|| value.get("model_name"))
        .and_then(Value::as_str)
        .map(str::to_owned)
        .or_else(|| {
            value
                .get("model")
                .and_then(|model| model.get("name"))
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
}
