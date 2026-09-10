use super::{read_events_for_list_with, read_events_with};
use crate::measure::store::ManagedPath;
use serde_json::json;
use std::fs::OpenOptions;
use std::io::Write;

#[test]
fn keeps_the_event_lock_while_reading_the_result_snapshot() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("events.jsonl");
    write_event(&path)?;

    let (_, result) = read_events_with(&ManagedPath::test_path(&path)?, &"b".repeat(64), || {
        let other = OpenOptions::new().read(true).write(true).open(&path)?;
        assert!(other.try_lock().is_err());
        Ok(7)
    })?;

    assert_eq!(result, 7);
    Ok(())
}

#[test]
fn list_keeps_the_event_lock_while_reading_the_result_snapshot()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("events.jsonl");
    write_event(&path)?;

    let result =
        read_events_for_list_with(&ManagedPath::test_path(&path)?, &"b".repeat(64), || {
            let other = OpenOptions::new().read(true).write(true).open(&path)?;
            assert!(other.try_lock().is_err());
            Ok(7)
        });

    assert_eq!(result?.1, 7);
    Ok(())
}

fn write_event(path: &std::path::Path) -> std::io::Result<()> {
    let mut file = std::fs::File::create(path)?;
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
    )?;
    Ok(())
}
