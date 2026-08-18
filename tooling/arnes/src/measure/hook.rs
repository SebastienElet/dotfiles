use super::MeasureError;
use super::fingerprint;
use super::input::Payload;
use super::model::{EventRecord, HookAgent, PromptRecord, RepositoryRecord, RunRecord};
use super::redaction::{redact, redact_string};
use super::repository;
use super::store::{Store, append_jsonl, write_json_atomic, write_json_once};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use std::env;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static EVENT_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub fn capture(agent: HookAgent) -> Result<(), MeasureError> {
    let observed = env::current_dir()?;
    let protected_root = repository::protected_root(&observed);
    let repository = repository::observe(&protected_root);
    let store = Store::open(&protected_root)?;
    let payload = Payload::read(&store, agent)?;
    let session = required_string(payload.value(), agent.session_key())
        .map(str::to_owned)
        .inspect_err(|error| {
            let _ = payload.record_invalid(&store, agent, &error.to_string());
        })?;
    persist_hook(
        &store,
        agent,
        &session,
        payload.into_value(),
        repository,
        &protected_root,
    )
}

fn persist_hook(
    store: &Store,
    agent: HookAgent,
    session: &str,
    raw: Value,
    repository: Option<RepositoryRecord>,
    deployment_root: &Path,
) -> Result<(), MeasureError> {
    let timestamp_ms = now_ms();
    let run_id = digest(&[agent.as_str().as_bytes(), session.as_bytes()]);
    let run_dir = store.run_dir(&run_id)?;
    let run = build_run(
        agent,
        session,
        run_id,
        timestamp_ms,
        &raw,
        repository,
        deployment_root,
    )?;
    write_json_once(&run_dir.join("run.json"), &run)?;
    let event_id = event_id(agent, session, &raw);
    let mut artifact = raw.clone();
    redact(&mut artifact);
    let artifact_path = format!("artifacts/hooks/{event_id}.json");
    write_json_atomic(&run_dir.join(&artifact_path), &artifact)?;
    append_event(&run_dir, timestamp_ms, &event_id, artifact_path, &raw)?;
    append_prompt(&run_dir, timestamp_ms, &event_id, session, &raw)?;
    Ok(())
}

fn build_run(
    agent: HookAgent,
    session: &str,
    run_id: String,
    timestamp_ms: u64,
    raw: &Value,
    repository: Option<RepositoryRecord>,
    deployment_root: &Path,
) -> Result<RunRecord, MeasureError> {
    let home = env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| MeasureError::new("HOME is required for harness fingerprinting"))?;
    Ok(RunRecord {
        schema_version: 1,
        run_id,
        agent: agent.as_str(),
        session_id: session.to_owned(),
        started_at_ms: timestamp_ms,
        model: model(raw).map(|model| redact_string(&model)),
        repository,
        harness_fingerprint: fingerprint::deployed(agent, &home, deployment_root)?,
    })
}

fn append_event(
    run_dir: &Path,
    timestamp_ms: u64,
    event_id: &str,
    artifact: String,
    raw: &Value,
) -> Result<(), MeasureError> {
    let event = EventRecord {
        timestamp_ms,
        event_id: event_id.to_owned(),
        event: event_name(raw),
        artifact,
        native_ids: native_ids(raw),
    };
    append_jsonl(&run_dir.join("events.jsonl"), &event)
}

fn append_prompt(
    run_dir: &Path,
    timestamp_ms: u64,
    event_id: &str,
    session: &str,
    raw: &Value,
) -> Result<(), MeasureError> {
    let Some(prompt) = raw.get("prompt").and_then(Value::as_str) else {
        return Ok(());
    };
    let prompt = PromptRecord {
        timestamp_ms,
        event_id: event_id.to_owned(),
        session_id: session.to_owned(),
        prompt_id: prompt_id(raw),
        prompt: redact_string(prompt),
    };
    append_jsonl(&run_dir.join("prompts.jsonl"), &prompt)
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

fn event_name(value: &Value) -> String {
    let event = ["hook_event_name", "event", "type"]
        .iter()
        .find_map(|key| value.get(key).and_then(Value::as_str))
        .unwrap_or("unknown");
    redact_string(event)
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

fn native_ids(value: &Value) -> Map<String, Value> {
    let mut ids = Map::new();
    for key in [
        "event_id",
        "hook_event_id",
        "request_id",
        "message_id",
        "turn_id",
        "generation_id",
    ] {
        if let Some(value) = value.get(key) {
            match value {
                Value::String(value) => {
                    ids.insert(key.to_owned(), Value::String(redact_string(value)));
                }
                Value::Number(_) => {
                    ids.insert(key.to_owned(), value.clone());
                }
                _ => {}
            }
        }
    }
    ids
}

fn prompt_id(value: &Value) -> Option<String> {
    [
        "prompt_id",
        "message_id",
        "turn_id",
        "generation_id",
        "request_id",
    ]
    .iter()
    .find_map(|key| value.get(key))
    .and_then(|value| match value {
        Value::String(value) => Some(redact_string(value)),
        Value::Number(value) => Some(value.to_string()),
        _ => None,
    })
}

fn event_id(agent: HookAgent, session: &str, value: &Value) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
        .to_string();
    let process = std::process::id().to_string();
    let sequence = EVENT_SEQUENCE.fetch_add(1, Ordering::Relaxed).to_string();
    let raw = serde_json::to_vec(value).unwrap_or_default();
    digest(&[
        agent.as_str().as_bytes(),
        session.as_bytes(),
        now.as_bytes(),
        process.as_bytes(),
        sequence.as_bytes(),
        &raw,
    ])
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
