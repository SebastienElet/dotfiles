use super::MeasureError;
use super::fingerprint;
use super::model::{HookAgent, RunRecord, RunRecordV2};
use super::repository;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use std::path::{Path, PathBuf};

pub struct NewRun<'a> {
    pub agent: HookAgent,
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
        .map(|_| repository::observe(input.observed));
    Ok(RunRecord::V2(RunRecordV2 {
        schema_version: 2,
        run_id: input.run_id,
        agent: input.agent.as_str().to_owned(),
        started_at_ms: input.timestamp_ms,
        model_fingerprint: model(input.raw).map(|model| {
            let digest = Sha256::digest(model.as_bytes());
            format!("{digest:x}")
        }),
        repository_commit: repository.as_ref().and_then(|value| value.head.clone()),
        repository_dirty: repository.map(|value| value.dirty),
        harness_fingerprint: fingerprint.digest,
        harness_fingerprint_limitations: fingerprint.limitations,
        operating_system: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
    }))
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
