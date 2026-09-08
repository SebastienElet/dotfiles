use super::super::MeasureError;
use super::super::store::{ManagedPath, Store};
use super::visit_events;
use std::fs::File;

const RETENTION_MS: u64 = 90 * 86_400_000;

struct ExpiredPr {
    path: ManagedPath,
    _lifecycle: File,
}

pub(in crate::measure) fn candidates(
    store: &Store,
    now_ms: u64,
) -> Result<Vec<String>, MeasureError> {
    let timelines = store.state_path("pull-requests");
    if !timelines.exists()? {
        return Ok(Vec::new());
    }
    let mut names = timelines.read_dir_names()?;
    names.sort();
    let mut candidates = Vec::new();
    for name in names {
        let pr_hash = name
            .into_string()
            .map_err(|_| MeasureError::new("managed PR identity hash is not UTF-8"))?;
        if expired_pr(store, &pr_hash, now_ms)?.is_some() {
            candidates.push(pr_hash);
        }
    }
    Ok(candidates)
}

pub(in crate::measure) fn remove_candidates(
    store: &Store,
    candidates: &[String],
    now_ms: u64,
    removed: &mut u64,
) -> Result<(), MeasureError> {
    for pr_hash in candidates {
        if let Some(pr) = expired_pr(store, pr_hash, now_ms)? {
            pr.path.remove_tree()?;
            *removed += 1;
        }
    }
    Ok(())
}

fn expired_pr(
    store: &Store,
    pr_hash: &str,
    now_ms: u64,
) -> Result<Option<ExpiredPr>, MeasureError> {
    let lock = store.open_pr_lock(pr_hash)?;
    lock.lock()?;
    let path = store.state_path("pull-requests").join(pr_hash);
    if !path.exists()? {
        return Ok(None);
    }
    path.open_directory()?;
    let now_ms = now_ms.max(super::super::hook::now_ms());
    let mut last_event = None;
    visit_events(&path.join("events.jsonl"), pr_hash, |record| {
        if record.timestamp_ms > now_ms {
            return Err(MeasureError::new(
                "retention refuses future PR event timestamps",
            ));
        }
        last_event = Some(last_event.map_or(record.timestamp_ms, |last: u64| {
            last.max(record.timestamp_ms)
        }));
        Ok(())
    })?;
    let last_event = last_event
        .ok_or_else(|| MeasureError::new("retention refuses a PR timeline without events"))?;
    if now_ms.saturating_sub(last_event) < RETENTION_MS {
        return Ok(None);
    }
    path.validate_removal()?;
    Ok(Some(ExpiredPr {
        path,
        _lifecycle: lock,
    }))
}
