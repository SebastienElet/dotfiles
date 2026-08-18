use super::{append_then_commit, preflight};
use crate::measure::MeasureError;
use crate::measure::result::{FinishArgs, MergeReady};
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

fn args_with_evidence(size: usize) -> FinishArgs {
    FinishArgs {
        run_id: "a".repeat(64),
        merge_ready: MergeReady::Pass,
        human_minutes: 1.0,
        human_edited_diff: false,
        failure_reason: None,
        evidence: vec!["x".repeat(size)],
        regression: false,
        invariant: Vec::new(),
    }
}

#[test]
fn result_preflight_accepts_a_payload_near_the_reader_limit() {
    preflight(&args_with_evidence(1_099_000)).unwrap();
}

#[test]
fn result_preflight_rejects_an_oversized_payload() {
    let error = preflight(&args_with_evidence(1_100_000)).unwrap_err();

    assert!(error.to_string().contains("exceeds 1100000 bytes"));
}
