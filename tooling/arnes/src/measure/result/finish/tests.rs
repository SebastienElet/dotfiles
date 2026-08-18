use super::append_then_commit;
use crate::measure::MeasureError;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom};

#[test]
fn commit_failure_truncates_the_appended_event_to_its_previous_offset() {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("events.jsonl");
    std::fs::write(&path, b"{\"event\":\"initial\"}\n").unwrap();
    let mut file = OpenOptions::new()
        .read(true)
        .append(true)
        .open(&path)
        .unwrap();
    file.lock().unwrap();

    let result = append_then_commit(&mut file, b"{\"event\":\"result_recorded\"}\n", || {
        Err(MeasureError::new("injected commit failure"))
    });

    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("injected commit failure")
    );
    file.seek(SeekFrom::Start(0)).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    assert_eq!(content, "{\"event\":\"initial\"}\n");
}
