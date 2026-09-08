use super::{
    Store, append_jsonl, append_jsonl_bytes, write_json_atomic_bytes, write_json_atomic_test,
};
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::sync::{Arc, Barrier};

const MAX_RECORD_BYTES: usize = 1_100_000;

#[test]
fn retention_waits_for_run_initialization_before_reading_metadata() {
    use crate::measure::{hook::now_ms, retention};
    use serde_json::json;
    use std::sync::mpsc::{self, RecvTimeoutError};
    use std::time::Duration;

    let directory = tempfile::tempdir().unwrap();
    let store = Store::open_from_state_base(directory.path(), &[]).unwrap();
    let run_id = "ab".repeat(32);
    let lifecycle = store.open_run_lock(&run_id).unwrap();
    lifecycle.lock().unwrap();
    let run = store.run_dir(&run_id).unwrap();
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        sender.send(retention::retain(&store, now_ms())).unwrap();
    });
    let before_initialization = receiver.recv_timeout(Duration::from_secs(1));
    let metadata = json!({
        "schema_version": 2, "run_id": run_id, "agent": "codex",
        "started_at_ms": now_ms(), "model_fingerprint": null,
        "repository_commit": null, "repository_dirty": null,
        "harness_fingerprint": "00".repeat(32), "harness_fingerprint_limitations": [],
        "operating_system": "macos", "architecture": "aarch64"
    });
    super::write_json_atomic(&run.join("run.json"), &metadata).unwrap();
    drop(lifecycle);
    worker.join().unwrap();
    assert!(
        matches!(before_initialization, Err(RecvTimeoutError::Timeout)),
        "retention read an uninitialized run: {before_initialization:?}"
    );
    receiver
        .recv_timeout(Duration::from_secs(5))
        .unwrap()
        .unwrap();
    assert!(run.join("run.json").exists().unwrap());
}

#[test]
fn pr_retention_rechecks_a_candidate_after_a_fresh_append() {
    use crate::measure::{hook::now_ms, pr_timeline::retention};
    use serde_json::json;

    let directory = tempfile::tempdir().unwrap();
    let store = Store::open_from_state_base(directory.path(), &[]).unwrap();
    let pr_hash = "ca2553be5531ecdb5e2cffad537225c437eeb107490fb7e44a189a38b3869421";
    let timeline = store.state_path("pull-requests").join(pr_hash);
    timeline.create_dir_all().unwrap();
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
    append_jsonl_bytes(&path, &super::jsonl_bytes(&event).unwrap()).unwrap();
    let candidates = retention::candidates(&store, now).unwrap();
    assert_eq!(candidates, [pr_hash]);

    let lock = store.open_pr_lock(pr_hash).unwrap();
    lock.lock().unwrap();
    event["timestamp_ms"] = json!(now_ms());
    event["head_sha"] = json!("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb");
    append_jsonl_bytes(&path, &super::jsonl_bytes(&event).unwrap()).unwrap();
    drop(lock);

    let mut removed = 0;
    retention::remove_candidates(&store, &candidates, now_ms(), &mut removed).unwrap();
    assert_eq!(removed, 0);
    assert!(path.exists().unwrap());
}

#[test]
fn jsonl_writer_accepts_the_readers_exact_line_limit() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("events.jsonl");
    let value = "x".repeat(MAX_RECORD_BYTES - 3);

    append_jsonl(&path, &value).unwrap();

    assert_eq!(fs::metadata(path).unwrap().len(), MAX_RECORD_BYTES as u64);
}

#[test]
fn jsonl_writer_rejects_an_oversized_line_without_mutation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("feedback.jsonl");
    let value = "x".repeat(MAX_RECORD_BYTES - 2);

    let error = append_jsonl(&path, &value).unwrap_err();

    assert!(error.to_string().contains("exceeds 1100000 bytes"));
    assert!(!path.exists());
}

#[test]
fn jsonl_writer_preserves_an_existing_file_when_the_new_line_is_oversized() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("prompts.jsonl");
    append_jsonl(&path, &"seed").unwrap();
    let before = fs::read(&path).unwrap();
    let value = "x".repeat(MAX_RECORD_BYTES - 2);

    assert!(append_jsonl(&path, &value).is_err());

    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn atomic_json_writer_rejects_an_oversized_file_without_mutation() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("result.json");
    let value = "x".repeat(MAX_RECORD_BYTES - 1);

    let error = write_json_atomic_test(&path, &value).unwrap_err();

    assert!(error.to_string().contains("exceeds 1100000 bytes"));
    assert!(!path.exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
}

#[test]
fn opened_store_anchors_new_existing_and_invalid_writes() {
    let fixture = tempfile::tempdir().unwrap();
    let external = fixture.path().join("external");
    let repository = fixture.path().join("repository");
    let state_link = fixture.path().join("state");
    let root = external.join("dotfiles/agent-harness");
    fs::create_dir_all(root.join("runs/existing")).unwrap();
    fs::create_dir_all(repository.join("dotfiles/agent-harness/runs/existing")).unwrap();
    symlink(&external, &state_link).unwrap();
    let store =
        Store::open_from_state_base(&state_link, std::slice::from_ref(&repository)).unwrap();
    fs::remove_file(&state_link).unwrap();
    symlink(&repository, &state_link).unwrap();

    let existing = store.run_dir("existing").unwrap();
    write_json_atomic_bytes(&existing.join("run.json"), b"{}\n").unwrap();
    append_jsonl_bytes(&existing.join("events.jsonl"), b"{}\n").unwrap();
    let new = store.run_dir("new").unwrap();
    write_json_atomic_bytes(&new.join("run.json"), b"{}\n").unwrap();
    store
        .append_invalid(&serde_json::json!({"safe":true}))
        .unwrap();

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
}

#[test]
fn concurrent_append_creation_never_loses_its_open_parent() {
    let directory = tempfile::tempdir().unwrap();
    for round in 0..64 {
        let path = super::ManagedPath::test_path(&directory.path().join(format!("{round}.jsonl")));
        let barrier = Arc::new(Barrier::new(24));
        let threads: Vec<_> = (0..24)
            .map(|_| {
                let path = path.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    let mut file = super::open_private_append(&path).unwrap();
                    file.write_all(b"{}\n").unwrap();
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }
}

#[test]
fn run_lock_survives_the_removal_of_its_run_directory() {
    let fixture = tempfile::tempdir().unwrap();
    let store = Store::open_from_state_base(fixture.path(), &[]).unwrap();
    let run_id = "a".repeat(64);
    let run = store.run_dir(&run_id).unwrap();
    let first = store.open_run_lock(&run_id).unwrap();
    first.lock().unwrap();

    run.remove_tree().unwrap();

    let second = store.open_run_lock(&run_id).unwrap();
    assert!(second.try_lock().is_err());
}
