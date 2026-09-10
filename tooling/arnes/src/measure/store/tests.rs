use super::{
    Store, append_jsonl, append_jsonl_bytes, write_json_atomic_bytes, write_json_atomic_test,
};
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::sync::{Arc, Barrier};

const MAX_RECORD_BYTES: usize = 1_100_000;

#[test]
fn retention_waits_for_run_initialization_before_reading_metadata()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::measure::{hook::now_ms, retention};
    use serde_json::json;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    let directory = tempfile::tempdir()?;
    let store = Store::open_from_state_base(directory.path(), &[])?;
    let run_id = "ab".repeat(32);
    let lifecycle = store.open_run_lock(&run_id)?;
    lifecycle.lock()?;
    let run = store.run_dir(&run_id)?;
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || sender.send(retention::retain(&store, now_ms())));
    let before_initialization = receiver.recv_timeout(Duration::from_secs(1));
    let metadata = json!({
        "schema_version": 2, "run_id": run_id, "agent": "codex",
        "started_at_ms": now_ms(), "model_fingerprint": null,
        "repository_commit": null, "repository_dirty": null,
        "harness_fingerprint": "00".repeat(32), "harness_fingerprint_limitations": [],
        "operating_system": "macos", "architecture": "aarch64"
    });
    super::write_json_atomic(&run.join("run.json"), &metadata)?;
    drop(lifecycle);
    worker.join().map_err(|_| "retention worker panicked")??;
    assert!(
        matches!(before_initialization, Err(RecvTimeoutError::Timeout)),
        "retention read an uninitialized run: {before_initialization:?}"
    );
    receiver.recv_timeout(Duration::from_secs(5))??;
    assert!(run.join("run.json").exists()?);
    Ok(())
}

#[test]
fn pr_retention_rechecks_a_candidate_after_a_fresh_append() -> Result<(), Box<dyn std::error::Error>>
{
    use crate::measure::{hook::now_ms, pr_timeline::retention};
    use serde_json::json;

    let directory = tempfile::tempdir()?;
    let store = Store::open_from_state_base(directory.path(), &[])?;
    let pr_hash = "ca2553be5531ecdb5e2cffad537225c437eeb107490fb7e44a189a38b3869421";
    let timeline = store.state_path("pull-requests").join(pr_hash);
    timeline.create_dir_all()?;
    let path = timeline.join("events.jsonl");
    let now = now_ms();
    let mut event = json!({
        "schema_version": 1, "event_type": "pr-verdict",
        "timestamp_ms": now - 91 * 86_400_000,
        "pr": {"forge": "github.com", "repository": "example/project", "pr_id": 1042},
        "head_sha": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "agent": "codex", "operating_system": "macos", "architecture": "aarch64",
        "data": {"verdict": "changes-required", "blocking_findings": 2,
                 "non_blocking_findings": 1, "observable_behaviors": 3, "evidence_gaps": 2}
    });
    append_jsonl_bytes(&path, &super::jsonl_bytes(&event)?)?;
    let candidates = retention::candidates(&store, now)?;
    assert_eq!(candidates, [pr_hash]);

    let lock = store.open_pr_lock(pr_hash)?;
    lock.lock()?;
    *event.get_mut("timestamp_ms").ok_or("missing timestamp")? = json!(now_ms());
    *event.get_mut("head_sha").ok_or("missing head")? =
        json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    append_jsonl_bytes(&path, &super::jsonl_bytes(&event)?)?;
    drop(lock);

    let mut removed = 0;
    retention::remove_candidates(&store, &candidates, now_ms(), &mut removed)?;
    assert_eq!(removed, 0);
    assert!(path.exists()?);
    Ok(())
}

#[test]
fn jsonl_writer_accepts_the_readers_exact_line_limit() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("events.jsonl");
    let value = "x".repeat(MAX_RECORD_BYTES - 3);

    append_jsonl(&path, &value)?;

    assert_eq!(fs::metadata(path)?.len(), MAX_RECORD_BYTES as u64);
    Ok(())
}

#[test]
fn jsonl_writer_rejects_an_oversized_line_without_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("feedback.jsonl");
    let value = "x".repeat(MAX_RECORD_BYTES - 2);

    let error = append_jsonl(&path, &value)
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("exceeds 1100000 bytes"));
    assert!(!path.exists());
    Ok(())
}

#[test]
fn jsonl_writer_preserves_an_existing_file_when_the_new_line_is_oversized()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("prompts.jsonl");
    append_jsonl(&path, &"seed")?;
    let before = fs::read(&path)?;
    let value = "x".repeat(MAX_RECORD_BYTES - 2);

    assert!(append_jsonl(&path, &value).is_err());

    assert_eq!(fs::read(path)?, before);
    Ok(())
}

#[test]
fn atomic_json_writer_rejects_an_oversized_file_without_mutation()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("result.json");
    let value = "x".repeat(MAX_RECORD_BYTES - 1);

    let error = write_json_atomic_test(&path, &value)
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("exceeds 1100000 bytes"));
    assert!(!path.exists());
    assert_eq!(fs::read_dir(directory.path())?.count(), 0);
    Ok(())
}

#[test]
fn opened_store_anchors_new_existing_and_invalid_writes() -> Result<(), Box<dyn std::error::Error>>
{
    let fixture = tempfile::tempdir()?;
    let external = fixture.path().join("external");
    let repository = fixture.path().join("repository");
    let state_link = fixture.path().join("state");
    let root = external.join("dotfiles/agent-harness");
    fs::create_dir_all(root.join("runs/existing"))?;
    fs::create_dir_all(repository.join("dotfiles/agent-harness/runs/existing"))?;
    symlink(&external, &state_link)?;
    let store = Store::open_from_state_base(&state_link, std::slice::from_ref(&repository))?;
    fs::remove_file(&state_link)?;
    symlink(&repository, &state_link)?;

    let existing = store.run_dir("existing")?;
    write_json_atomic_bytes(&existing.join("run.json"), b"{}\n")?;
    append_jsonl_bytes(&existing.join("events.jsonl"), b"{}\n")?;
    let new = store.run_dir("new")?;
    write_json_atomic_bytes(&new.join("run.json"), b"{}\n")?;
    store.append_invalid(&serde_json::json!({"safe":true}))?;

    assert!(root.join("runs/existing/run.json").is_file());
    assert!(root.join("runs/existing/events.jsonl").is_file());
    assert!(root.join("runs/new/run.json").is_file());
    assert!(root.join("invalid.jsonl").is_file());
    assert!(!repository.join("dotfiles/agent-harness/runs/new").exists());
    assert!(
        !repository
            .join("dotfiles/agent-harness/invalid.jsonl")
            .exists()
    );
    Ok(())
}

#[test]
fn concurrent_append_creation_never_loses_its_open_parent() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = tempfile::tempdir()?;
    for round in 0..64 {
        let path = super::ManagedPath::test_path(&directory.path().join(format!("{round}.jsonl")))?;
        let barrier = Arc::new(Barrier::new(24));
        let threads: Vec<_> = (0..24)
            .map(|_| {
                let path = path.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || -> Result<(), crate::measure::MeasureError> {
                    barrier.wait();
                    let mut file = super::open_private_append(&path)?;
                    file.write_all(b"{}\n")?;
                    Ok(())
                })
            })
            .collect();
        for thread in threads {
            thread.join().map_err(|_| "append worker panicked")??;
        }
    }
    Ok(())
}

#[test]
fn run_lock_survives_the_removal_of_its_run_directory() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = tempfile::tempdir()?;
    let store = Store::open_from_state_base(fixture.path(), &[])?;
    let run_id = "a".repeat(64);
    let run = store.run_dir(&run_id)?;
    let first = store.open_run_lock(&run_id)?;
    first.lock()?;

    run.remove_tree()?;

    let second = store.open_run_lock(&run_id)?;
    assert!(second.try_lock().is_err());
    Ok(())
}
