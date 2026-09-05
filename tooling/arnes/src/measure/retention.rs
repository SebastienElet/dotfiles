use super::MeasureError;
use super::outcome::latest as latest_outcome;
use super::result::{open_run, read_events_for_list_with, read_optional_json};
use super::store::{Store, open_private_append, validation, write_json_atomic};
use serde::{Deserialize, Serialize};
use std::fs::File;

const DAY_MS: u64 = 86_400_000;
const RETENTION_MS: u64 = 60 * DAY_MS;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RetentionState {
    schema_version: u8,
    status: RetentionStatus,
    swept_at_ms: u64,
    next_sweep_at_ms: u64,
    candidate_runs: u64,
    removed_runs: u64,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum RetentionStatus {
    Sweeping,
    Complete,
    Failed,
}

struct ExpiredRun {
    path: super::store::ManagedPath,
    _lifecycle: File,
}

pub fn retain(store: &Store, now_ms: u64) -> Result<(), MeasureError> {
    let state_path = store.state_path("retention.json");
    if read_state(&state_path)?.is_some_and(|state| suppresses_sweep(&state, now_ms)) {
        return Ok(());
    }
    let lock = open_private_append(&store.state_path("retention.lock"))?;
    lock.lock()?;
    if read_state(&state_path)?.is_some_and(|state| suppresses_sweep(&state, now_ms)) {
        return Ok(());
    }
    let candidates = match candidates(store, now_ms) {
        Ok(candidates) => candidates,
        Err(error) => {
            write_state(&state_path, RetentionStatus::Failed, now_ms, 0, 0)?;
            return Err(error);
        }
    };
    let candidate_runs = u64::try_from(candidates.len())
        .map_err(|_| MeasureError::new("retention candidate count overflow"))?;
    write_state(
        &state_path,
        RetentionStatus::Sweeping,
        now_ms,
        candidate_runs,
        0,
    )?;
    let removed_runs = remove_candidates(store, &candidates, now_ms, &state_path)?;
    write_state(
        &state_path,
        RetentionStatus::Complete,
        now_ms,
        candidate_runs,
        removed_runs,
    )
}

fn candidates(store: &Store, now_ms: u64) -> Result<Vec<String>, MeasureError> {
    let runs = store.runs_path();
    if !runs.exists()? {
        return Ok(Vec::new());
    }
    runs.open_directory()?;
    let mut names = runs.read_dir_names()?;
    names.sort();
    let mut candidates = Vec::new();
    for name in names {
        let run_id = name
            .into_string()
            .map_err(|_| MeasureError::new("managed run id is not UTF-8"))?;
        if expired_run(store, &run_id, now_ms)?.is_some() {
            candidates.push(run_id);
        }
    }
    Ok(candidates)
}

fn remove_candidates(
    store: &Store,
    candidates: &[String],
    now_ms: u64,
    state_path: &super::store::ManagedPath,
) -> Result<u64, MeasureError> {
    let mut removed = 0_u64;
    for run_id in candidates {
        let run = match expired_run(store, run_id, now_ms) {
            Ok(run) => run,
            Err(error) => {
                write_state(
                    state_path,
                    RetentionStatus::Failed,
                    now_ms,
                    u64::try_from(candidates.len()).unwrap_or(u64::MAX),
                    removed,
                )?;
                return Err(error);
            }
        };
        if let Some(run) = run {
            if let Err(error) = run.path.remove_tree() {
                write_state(
                    state_path,
                    RetentionStatus::Failed,
                    now_ms,
                    u64::try_from(candidates.len()).unwrap_or(u64::MAX),
                    removed,
                )?;
                return Err(error);
            }
            removed = removed
                .checked_add(1)
                .ok_or_else(|| MeasureError::new("retention removal count overflow"))?;
        }
    }
    Ok(removed)
}

fn expired_run(
    store: &Store,
    run_id: &str,
    now_ms: u64,
) -> Result<Option<ExpiredRun>, MeasureError> {
    let run = open_run(store, run_id)?;
    let metadata = validation::read_run(&run.join("run.json"))?;
    if metadata.schema_version() != 2 {
        return Ok(None);
    }
    let lock = store.open_run_lock(run_id)?;
    lock.lock()?;
    let now_ms = now_ms.max(super::hook::now_ms());
    let metadata = validation::read_run(&run.join("run.json"))?;
    let (events, ()) = read_events_for_list_with(&run.join("events.jsonl"), run_id, || Ok(()))?;
    if !events.timestamps_consistent() {
        return Err(MeasureError::new(
            "retention refuses incoherent event timestamps",
        ));
    }
    let last_event = events
        .last_event_at_ms()
        .unwrap_or_else(|| metadata.started_at_ms());
    let last_observed = latest_outcome(&run.join("outcomes.jsonl"), run_id)?
        .map_or(last_event, |outcome| {
            last_event.max(outcome.recorded_at_ms())
        });
    if last_observed < metadata.started_at_ms() || last_observed > now_ms {
        return Err(MeasureError::new(
            "retention refuses timestamps outside the run interval",
        ));
    }
    if now_ms.saturating_sub(last_observed) < RETENTION_MS {
        return Ok(None);
    }
    run.validate_removal()?;
    Ok(Some(ExpiredRun {
        path: run,
        _lifecycle: lock,
    }))
}

fn write_state(
    path: &super::store::ManagedPath,
    status: RetentionStatus,
    now_ms: u64,
    candidate_runs: u64,
    removed_runs: u64,
) -> Result<(), MeasureError> {
    write_json_atomic(
        path,
        &RetentionState {
            schema_version: 1,
            status,
            swept_at_ms: now_ms,
            next_sweep_at_ms: now_ms.saturating_add(DAY_MS),
            candidate_runs,
            removed_runs,
        },
    )
}

fn suppresses_sweep(state: &RetentionState, now_ms: u64) -> bool {
    state.status != RetentionStatus::Sweeping && state.next_sweep_at_ms > now_ms
}

fn read_state(path: &super::store::ManagedPath) -> Result<Option<RetentionState>, MeasureError> {
    let state = read_optional_json(path, "retention.json")?;
    if state.as_ref().is_some_and(|state: &RetentionState| {
        state.schema_version != 1
            || state.next_sweep_at_ms < state.swept_at_ms
            || state.removed_runs > state.candidate_runs
    }) {
        return Err(MeasureError::new(
            "managed retention.json has an invalid record",
        ));
    }
    Ok(state)
}
