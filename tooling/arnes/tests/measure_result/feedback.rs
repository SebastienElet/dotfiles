use super::measure_support::*;
use serde_json::json;
use std::fs;

#[test]
fn feedback_preserves_private_multiline_text_and_never_mutates_result() {
    let harness = Harness::new();
    let run_id = harness.capture("claude-code", "session_id", "session", "prompt");
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]));
    let result_before = fs::read(harness.run_path(&run_id).join("result.json")).unwrap();
    let output = harness.run(&[
        "measure",
        "feedback",
        &run_id,
        "--source-type",
        "human",
        "--source-id",
        "reviewer@example.test",
        "--scope",
        "diff",
        "--observed",
        "private line one\nprivate line two",
        "--expected",
        "handle the empty response",
        "--evidence",
        "review-thread-1",
        "--invariant",
        "empty-response-safe",
        "--severity",
        "blocking",
        "--adjudication",
        "confirmed",
        "--resolution",
        "open",
        "--failure-category",
        "correctness",
    ]);
    assert_success(&output);

    let feedback = read_jsonl(harness.run_path(&run_id).join("feedback.jsonl"));
    assert_eq!(feedback.len(), 1);
    assert_eq!(feedback[0]["source_type"], "human");
    assert_eq!(
        feedback[0]["observed"],
        "private line one\nprivate line two"
    );
    assert_eq!(feedback[0]["analysis_blocking"], true);
    assert_eq!(
        fs::read(harness.run_path(&run_id).join("result.json")).unwrap(),
        result_before
    );
}

#[test]
fn only_confirmed_blocking_feedback_is_marked_for_analysis() {
    let harness = Harness::new();
    let run_id = harness.capture("cursor", "conversation_id", "session", "prompt");
    for (severity, adjudication, expected) in [
        ("blocking", "pending", false),
        ("major", "confirmed", false),
        ("blocking", "confirmed", true),
    ] {
        let output = harness.run(&[
            "measure",
            "feedback",
            &run_id,
            "--source-type",
            "harness",
            "--source-id",
            "ci-review",
            "--scope",
            "change",
            "--observed",
            "behavior",
            "--expected",
            "other behavior",
            "--severity",
            severity,
            "--adjudication",
            adjudication,
            "--resolution",
            "open",
            "--failure-category",
            "requirements",
        ]);
        assert_success(&output);
        let feedback = read_jsonl(harness.run_path(&run_id).join("feedback.jsonl"));
        assert_eq!(feedback.last().unwrap()["analysis_blocking"], expected);
    }
}

#[test]
fn feedback_rejects_a_semantically_invalid_existing_log() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let path = harness.run_path(&run_id).join("feedback.jsonl");
    fs::write(&path, b"{\"schema_version\":1}\n").unwrap();
    let output = harness.run(&[
        "measure",
        "feedback",
        &run_id,
        "--source-type",
        "human",
        "--source-id",
        "reviewer",
        "--scope",
        "diff",
        "--observed",
        "observation",
        "--expected",
        "expectation",
        "--severity",
        "minor",
        "--adjudication",
        "pending",
        "--resolution",
        "open",
        "--failure-category",
        "maintainability",
    ]);
    assert_failure(&output, "feedback.jsonl has an invalid record");
    assert_eq!(fs::read_to_string(path).unwrap().lines().count(), 1);
}

#[test]
fn feedback_rejects_duplicate_fields_in_existing_records() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let path = harness.run_path(&run_id).join("feedback.jsonl");
    let record = json!({
        "schema_version": 1,
        "feedback_id": "a".repeat(64),
        "run_id": run_id,
        "recorded_at_ms": 1,
        "source_type": "human",
        "source_id": "first",
        "scope": "diff",
        "observed": "observation",
        "expected": "expectation",
        "evidence": [],
        "invariants": [],
        "severity": "minor",
        "adjudication": "pending",
        "resolution": "open",
        "failure_category": "maintainability",
        "analysis_blocking": false
    });
    let line = serde_json::to_string(&record).unwrap().replace(
        "\"source_id\":\"first\"",
        "\"source_id\":\"first\",\"source_id\":\"second\"",
    );
    fs::write(&path, format!("{line}\n")).unwrap();

    let output = harness.run(&[
        "measure",
        "feedback",
        record["run_id"].as_str().unwrap(),
        "--source-type",
        "human",
        "--source-id",
        "reviewer",
        "--scope",
        "diff",
        "--observed",
        "observation",
        "--expected",
        "expectation",
        "--severity",
        "minor",
        "--adjudication",
        "pending",
        "--resolution",
        "open",
        "--failure-category",
        "maintainability",
    ]);
    assert_failure(&output, "feedback.jsonl has an invalid record");
}

#[test]
fn parallel_feedback_appends_are_complete_and_unique() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let mut children = Vec::new();
    for index in 0..16 {
        children.push(
            harness
                .command()
                .args([
                    "measure",
                    "feedback",
                    &run_id,
                    "--source-type",
                    "human",
                    "--source-id",
                    &format!("reviewer-{index}"),
                    "--scope",
                    "diff",
                    "--observed",
                    "observation",
                    "--expected",
                    "expectation",
                    "--severity",
                    "minor",
                    "--adjudication",
                    "pending",
                    "--resolution",
                    "open",
                    "--failure-category",
                    "maintainability",
                ])
                .spawn()
                .unwrap(),
        );
    }
    for child in children {
        assert_success(&child.wait_with_output().unwrap());
    }
    let feedback = read_jsonl(harness.run_path(&run_id).join("feedback.jsonl"));
    assert_eq!(feedback.len(), 16);
    let mut ids: Vec<&str> = feedback
        .iter()
        .map(|item| item["source_id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 16);
}
