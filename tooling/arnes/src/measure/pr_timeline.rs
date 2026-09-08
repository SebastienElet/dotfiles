mod model;
pub(super) mod retention;
mod validation;

use super::MeasureError;
use super::hook::now_ms;
use super::result::{open_store, visit_jsonl_typed};
use super::store::{ManagedPath, Store, append_jsonl_bytes, jsonl_bytes};
pub use model::PrVerdictArgs;
use model::{EventType, PrEventRecord, PrIdentity, VerdictData};
use sha2::{Digest, Sha256};
use std::collections::HashSet;

pub fn record(args: PrVerdictArgs) -> Result<&'static str, MeasureError> {
    validation::input(&args)?;
    let store = open_store()?;
    let status = persist(&store, args)?;
    super::retention::retain(&store, now_ms()).map_err(|error| {
        MeasureError::new(format!("PR event {status}; retention failed: {error}"))
    })?;
    Ok(status)
}

fn persist(store: &Store, args: PrVerdictArgs) -> Result<&'static str, MeasureError> {
    let pr_hash = identity_hash(&args.pr)?;
    let lock = store.open_pr_lock(&pr_hash)?;
    lock.lock()?;
    let directory = store.state_path("pull-requests").join(&pr_hash);
    directory.create_dir_all()?;
    let path = directory.join("events.jsonl");
    if let Some(existing) = recorded_verdict(&path, &pr_hash, &args.head_sha)? {
        if existing == args.data {
            return Ok("duplicate");
        }
        return Err(MeasureError::new(
            "different verdict already recorded for this PR, SHA and event type; history is unchanged",
        ));
    }
    let event = PrEventRecord {
        schema_version: 1,
        event_type: EventType::PrVerdict,
        timestamp_ms: now_ms(),
        pr: args.pr,
        head_sha: args.head_sha,
        agent: args.agent.map(|agent| agent.as_str().to_owned()),
        operating_system: std::env::consts::OS.to_owned(),
        architecture: std::env::consts::ARCH.to_owned(),
        data: args.data,
    };
    validation::record(&event)?;
    append_jsonl_bytes(&path, &jsonl_bytes(&event)?)?;
    Ok("recorded")
}

fn recorded_verdict(
    path: &ManagedPath,
    pr_hash: &str,
    head_sha: &str,
) -> Result<Option<VerdictData>, MeasureError> {
    let mut existing = None;
    visit_events(path, pr_hash, |record| {
        if record.event_type == EventType::PrVerdict && record.head_sha == head_sha {
            existing = Some(record.data);
        }
        Ok(())
    })?;
    Ok(existing)
}

fn visit_events(
    path: &ManagedPath,
    pr_hash: &str,
    mut visitor: impl FnMut(PrEventRecord) -> Result<(), MeasureError>,
) -> Result<(), MeasureError> {
    let mut keys = HashSet::new();
    visit_jsonl_typed(path, "PR events.jsonl", |record: PrEventRecord| {
        validation::record(&record)?;
        if identity_hash(&record.pr)? != pr_hash
            || !keys.insert((record.event_type, record.head_sha.clone()))
        {
            return Err(MeasureError::new(
                "managed PR timeline has a conflicting identity or duplicate event",
            ));
        }
        visitor(record)
    })
}

fn identity_hash(pr: &PrIdentity) -> Result<String, MeasureError> {
    Ok(format!("{:x}", Sha256::digest(serde_json::to_vec(pr)?)))
}
