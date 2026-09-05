use super::{
    Store, append_jsonl, append_jsonl_bytes, write_json_atomic_bytes, write_json_atomic_test,
};
use std::fs;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::sync::{Arc, Barrier};

const MAX_RECORD_BYTES: usize = 1_100_000;

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
