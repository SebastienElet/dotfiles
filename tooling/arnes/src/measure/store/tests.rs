use super::{append_jsonl, write_json_atomic};
use std::fs;

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

    let error = write_json_atomic(&path, &value).unwrap_err();

    assert!(error.to_string().contains("exceeds 1100000 bytes"));
    assert!(!path.exists());
    assert_eq!(fs::read_dir(directory.path()).unwrap().count(), 0);
}
