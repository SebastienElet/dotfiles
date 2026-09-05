use super::measure_support::*;
use serde_json::json;
use std::fs;
use std::io::Write;

#[test]
fn finish_refuses_duplicate_result_revisions_without_mutation() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]));
    let events_path = run.join("events.jsonl");
    let decisions: Vec<String> = fs::read_to_string(&events_path)
        .unwrap()
        .lines()
        .filter(|line| line.contains("result_recorded"))
        .map(str::to_owned)
        .collect();
    assert_eq!(decisions.len(), 1);
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&events_path)
        .unwrap();
    writeln!(file, "{}", decisions[0]).unwrap();
    let events_before = fs::read(&events_path).unwrap();
    let result_before = fs::read(run.join("result.json")).unwrap();

    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "2",
    ]);
    assert_failure(&output, "result revisions must be unique and continuous");
    assert_eq!(fs::read(events_path).unwrap(), events_before);
    assert_eq!(fs::read(run.join("result.json")).unwrap(), result_before);
}

#[test]
fn finish_fails_closed_for_unknown_runs_and_malformed_managed_files() {
    let harness = Harness::new();
    let unknown = "0".repeat(64);
    assert_failure(
        &harness.run(&[
            "measure",
            "finish",
            &unknown,
            "--merge-ready",
            "pass",
            "--human-minutes",
            "0",
        ]),
        "unknown run",
    );
    assert!(!harness.run_path(&unknown).exists());

    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    fs::write(harness.run_path(&run_id).join("result.json"), b"not json").unwrap();
    assert_failure(
        &harness.run(&[
            "measure",
            "finish",
            &run_id,
            "--merge-ready",
            "pass",
            "--human-minutes",
            "0",
        ]),
        "result.json",
    );
}

#[test]
fn finish_does_not_publish_result_when_the_event_log_cannot_be_opened() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    fs::remove_file(run.join("events.jsonl")).unwrap();
    fs::create_dir(run.join("events.jsonl")).unwrap();

    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]);
    assert_failure(&output, "Is a directory");
    assert!(!run.join("result.json").exists());
}

#[test]
fn finish_rejects_an_invalid_event_record_without_mutating_the_run() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    fs::write(run.join("events.jsonl"), b"{}\n").unwrap();

    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]);
    assert_failure(&output, "events.jsonl has an invalid record");
    assert!(!run.join("result.json").exists());
    assert_eq!(fs::read(run.join("events.jsonl")).unwrap(), b"{}\n");
}

#[test]
fn finish_rejects_a_semantically_invalid_existing_result() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]));
    let path = harness.run_path(&run_id).join("result.json");
    let mut result = read_json(&path);
    result["revision"] = json!(0);
    fs::write(&path, serde_json::to_vec(&result).unwrap()).unwrap();

    assert_failure(
        &harness.run(&[
            "measure",
            "finish",
            &run_id,
            "--merge-ready",
            "pass",
            "--human-minutes",
            "1",
        ]),
        "invalid revision",
    );
}
