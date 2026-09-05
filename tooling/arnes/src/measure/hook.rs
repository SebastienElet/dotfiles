use super::MeasureError;
use super::events;
use super::input::Payload;
use super::model::HookAgent;
use super::repository;
use super::run;
use super::store::{Store, append_jsonl_bytes, jsonl_bytes, write_json_once};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::env;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn capture(agent: HookAgent) -> Result<(), MeasureError> {
    let observed = env::current_dir()?;
    let repository_root = repository::root(&observed);
    let protected_roots =
        repository::protected_roots(&observed, repository_root.as_ref().map(Path::new));
    let deployment_root = repository_root
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or_else(|| observed.clone());
    let store = Store::open(&protected_roots)?;
    let payload = Payload::read(&store, agent)?;
    let session = required_string(payload.value(), agent.session_key())
        .map(str::to_owned)
        .inspect_err(|error| {
            let _ = payload.record_invalid(&store, agent, &error.to_string());
        })?;
    let result = persist_hook(
        &store,
        agent,
        &session,
        payload.value(),
        repository_root,
        &observed,
        &deployment_root,
    );
    if let Err(error) = &result {
        let _ = payload.record_invalid(&store, agent, &error.to_string());
    }
    result?;
    super::retention::retain(&store, now_ms())
}

fn persist_hook(
    store: &Store,
    agent: HookAgent,
    session: &str,
    raw: &Value,
    repository_root: Option<String>,
    observed: &Path,
    deployment_root: &Path,
) -> Result<(), MeasureError> {
    let run_id = digest(&[agent.as_str().as_bytes(), session.as_bytes()]);
    let run_dir = store.run_dir(&run_id)?;
    let lifecycle = store.open_run_lock(&run_id)?;
    lifecycle.lock()?;
    let timestamp_ms = now_ms();
    let run_json = run_dir.join("run.json");
    let run = if run_json.exists()? {
        None
    } else {
        Some(run::build(run::NewRun {
            agent,
            run_id: run_id.clone(),
            timestamp_ms,
            raw,
            repository_root,
            observed,
            deployment_root,
        })?)
    };
    let event = events::record(timestamp_ms, raw);
    let event_bytes = jsonl_bytes(&event)?;
    write_json_once(&run_json, run.as_ref(), agent.as_str(), session, &run_id)?;
    append_jsonl_bytes(&run_dir.join("events.jsonl"), &event_bytes)?;
    Ok(())
}

fn required_string<'a>(value: &'a Value, key: &str) -> Result<&'a str, MeasureError> {
    value
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            MeasureError::new(format!("{key} is required and must be a non-empty string"))
        })
}

fn digest(parts: &[&[u8]]) -> String {
    let mut hasher = Sha256::new();
    for part in parts {
        hasher.update(part);
    }
    format!("{:x}", hasher.finalize())
}

pub(super) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}
