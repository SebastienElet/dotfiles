use super::measure_support::*;
use serde_json::{Value, json};
use std::fs;

#[test]
fn finish_recovers_a_missing_result_from_the_complete_event_snapshot() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "fail",
        "--human-minutes",
        "3",
        "--failure-reason",
        "first adjudication",
        "--evidence",
        "review-one",
    ]));
    fs::remove_file(run.join("result.json")).unwrap();

    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "5",
        "--evidence",
        "review-two",
    ]));

    let result = read_json(run.join("result.json"));
    assert_eq!(result["revision"], 2);
    assert_eq!(result["merge_ready"], "pass");
    let events = read_jsonl(run.join("events.jsonl"));
    let decisions: Vec<&Value> = events
        .iter()
        .filter(|event| event["event"] == "result_recorded")
        .collect();
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0]["result"]["revision"], 1);
    assert_eq!(decisions[0]["result"]["merge_ready"], "fail");
    assert_eq!(
        decisions[0]["result"]["failure_reason"],
        "first adjudication"
    );
    assert_eq!(decisions[1]["result"]["revision"], 2);
    assert_eq!(decisions[1]["result"]["merge_ready"], "pass");
}

#[test]
fn finish_recovers_when_result_is_exactly_one_revision_behind_history() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "fail",
        "--human-minutes",
        "1",
        "--failure-reason",
        "revision one",
    ]));
    let revision_one = fs::read(&result_path).unwrap();
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "unjudgeable",
        "--human-minutes",
        "2",
        "--evidence",
        "revision two evidence",
    ]));
    fs::write(&result_path, revision_one).unwrap();

    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "3",
    ]));

    let result = read_json(&result_path);
    assert_eq!(result["revision"], 3);
    assert_eq!(result["merge_ready"], "pass");
    let events = read_jsonl(run.join("events.jsonl"));
    let decisions: Vec<&Value> = events
        .iter()
        .filter(|event| event["event"] == "result_recorded")
        .collect();
    assert_eq!(decisions.len(), 3);
    assert_eq!(decisions[0]["result"]["failure_reason"], "revision one");
    assert_eq!(decisions[1]["result"]["merge_ready"], "unjudgeable");
    assert_eq!(
        decisions[1]["result"]["evidence"][0],
        "revision two evidence"
    );
    assert_eq!(decisions[2]["result"]["merge_ready"], "pass");
}

#[test]
fn finish_refuses_a_result_more_than_one_revision_behind_without_mutation() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    let mut revision_one = None;
    for minutes in 1..=3 {
        assert_success(&harness.run(&[
            "measure",
            "finish",
            &run_id,
            "--merge-ready",
            "pass",
            "--human-minutes",
            &minutes.to_string(),
        ]));
        if minutes == 1 {
            revision_one = Some(fs::read(&result_path).unwrap());
        }
    }
    fs::write(&result_path, revision_one.unwrap()).unwrap();
    let result_before = fs::read(&result_path).unwrap();
    let events_path = run.join("events.jsonl");
    let events_before = fs::read(&events_path).unwrap();

    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "4",
    ]);
    assert_failure(&output, "result.json diverges from result_recorded history");
    assert_eq!(fs::read(result_path).unwrap(), result_before);
    assert_eq!(fs::read(events_path).unwrap(), events_before);
}

#[test]
fn finish_refuses_result_and_event_history_divergence_without_mutation() {
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
    let result_path = run.join("result.json");
    let events_path = run.join("events.jsonl");
    let mut result = read_json(&result_path);
    result["human_minutes"] = json!(999.0);
    fs::write(&result_path, serde_json::to_vec(&result).unwrap()).unwrap();
    let result_before = fs::read(&result_path).unwrap();
    let events_before = fs::read(&events_path).unwrap();

    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "2",
    ]);
    assert_failure(&output, "result.json diverges from result_recorded history");
    assert_eq!(fs::read(result_path).unwrap(), result_before);
    assert_eq!(fs::read(events_path).unwrap(), events_before);
}

#[test]
fn list_refuses_a_result_more_than_one_revision_behind_without_mutation() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    let mut revision_one = None;
    for minutes in 1..=3 {
        assert_success(&harness.run(&[
            "measure",
            "finish",
            &run_id,
            "--merge-ready",
            "pass",
            "--human-minutes",
            &minutes.to_string(),
        ]));
        if minutes == 1 {
            revision_one = Some(fs::read(&result_path).unwrap());
        }
    }
    fs::write(&result_path, revision_one.unwrap()).unwrap();
    let result_before = fs::read(&result_path).unwrap();
    let events_path = run.join("events.jsonl");
    let events_before = fs::read(&events_path).unwrap();

    let output = harness.run(&["measure", "list", "--format", "json"]);

    assert_failure(&output, "result.json diverges from result_recorded history");
    assert_eq!(fs::read(result_path).unwrap(), result_before);
    assert_eq!(fs::read(events_path).unwrap(), events_before);
}
