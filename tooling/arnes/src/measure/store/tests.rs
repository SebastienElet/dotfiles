use super::{
    Store, append_jsonl, append_jsonl_bytes, write_json_atomic_bytes, write_json_atomic_test,
};
use std::fs;
use std::os::unix::fs::symlink;

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
    fs::create_dir_all(root.join("runs/existing/artifacts/hooks")).unwrap();
    fs::create_dir_all(repository.join("dotfiles/agent-harness/runs/existing/artifacts/hooks"))
        .unwrap();
    symlink(&external, &state_link).unwrap();
    let previous = std::env::var_os("XDG_STATE_HOME");
    unsafe { std::env::set_var("XDG_STATE_HOME", &state_link) };
    let store = Store::open(std::slice::from_ref(&repository)).unwrap();
    fs::remove_file(&state_link).unwrap();
    symlink(&repository, &state_link).unwrap();

    let existing = store.run_dir("existing").unwrap();
    write_json_atomic_bytes(&existing.join("artifacts/hooks/existing.json"), b"{}\n").unwrap();
    append_jsonl_bytes(&existing.join("prompts.jsonl"), b"{}\n").unwrap();
    let new = store.run_dir("new").unwrap();
    write_json_atomic_bytes(&new.join("artifacts/hooks/new.json"), b"{}\n").unwrap();
    store
        .append_invalid(&serde_json::json!({"safe":true}))
        .unwrap();

    restore_xdg(previous);
    assert!(
        root.join("runs/existing/artifacts/hooks/existing.json")
            .is_file()
    );
    assert!(root.join("runs/existing/prompts.jsonl").is_file());
    assert!(root.join("runs/new/artifacts/hooks/new.json").is_file());
    assert!(root.join("invalid.jsonl").is_file());
    assert!(!repository.join("dotfiles/agent-harness/runs/new").exists());
    assert!(
        !repository
            .join("dotfiles/agent-harness/invalid.jsonl")
            .exists()
    );
}

fn restore_xdg(previous: Option<std::ffi::OsString>) {
    match previous {
        Some(value) => unsafe { std::env::set_var("XDG_STATE_HOME", value) },
        None => unsafe { std::env::remove_var("XDG_STATE_HOME") },
    }
}
