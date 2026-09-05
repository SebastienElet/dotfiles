use super::{read_events_for_list_with, read_events_with};
use crate::measure::store::ManagedPath;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;

#[test]
fn keeps_the_event_lock_while_reading_the_result_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("events.jsonl");
    write_event(&path);

    let (_, result) = read_events_with(&ManagedPath::test_path(&path), &"b".repeat(64), || {
        let other = OpenOptions::new().read(true).write(true).open(&path)?;
        assert!(other.try_lock().is_err());
        Ok(7)
    })
    .unwrap();

    assert_eq!(result, 7);
}

#[test]
fn list_keeps_the_event_lock_while_reading_the_result_snapshot() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("events.jsonl");
    write_event(&path);

    let result = read_events_for_list_with(&ManagedPath::test_path(&path), &"b".repeat(64), || {
        let other = OpenOptions::new().read(true).write(true).open(&path)?;
        assert!(other.try_lock().is_err());
        Ok(7)
    });

    assert_eq!(result.unwrap().1, 7);
}

fn write_event(path: &std::path::Path) {
    let mut file = std::fs::File::create(path).unwrap();
    writeln!(
        file,
        "{}",
        json!({
            "timestamp_ms": 1,
            "event_id": "a".repeat(64),
            "event": "prompt.submit",
            "native_event": "UserPromptSubmit",
            "artifact": "artifacts/hooks/event.json",
            "native_ids": {}
        })
    )
    .unwrap();
}
