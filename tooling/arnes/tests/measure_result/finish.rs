use super::measure_support::*;
use serde_json::Value;

#[test]
fn finish_rejects_invalid_oracle_combinations() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    for (arguments, expected) in [
        (
            vec!["--merge-ready", "pass", "--human-minutes", "-1"],
            "finite non-negative",
        ),
        (
            vec!["--merge-ready", "pass", "--human-minutes", "NaN"],
            "finite non-negative",
        ),
        (
            vec!["--merge-ready", "fail", "--human-minutes", "1"],
            "failure reason is required",
        ),
        (
            vec![
                "--merge-ready",
                "pass",
                "--human-minutes",
                "1",
                "--failure-reason",
                "not actually ready",
            ],
            "failure reason is forbidden",
        ),
        (
            vec!["--merge-ready", "unjudgeable", "--human-minutes", "1"],
            "evidence is required",
        ),
    ] {
        let mut command = vec!["measure", "finish", &run_id];
        command.extend(arguments);
        assert_failure(&harness.run(&command), expected);
    }
    assert!(!harness.run_path(&run_id).join("result.json").exists());
}

#[test]
fn repeated_finish_increments_revision_and_keeps_each_adjudication_in_events() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let first = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "fail",
        "--human-minutes",
        "7",
        "--human-edited-diff",
        "--failure-reason",
        "tests failed",
        "--evidence",
        "cargo test",
        "--regression",
        "--invariant",
        "tests-green",
    ]);
    assert_success(&first);
    let second = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "9",
        "--evidence",
        "fixed and rerun",
        "--invariant",
        "tests-green",
    ]);
    assert_success(&second);

    let result = read_json(harness.run_path(&run_id).join("result.json"));
    assert_eq!(result["revision"], 2);
    assert_eq!(result["merge_ready"], "pass");
    assert_eq!(result["human_minutes"], 9.0);
    let events = read_jsonl(harness.run_path(&run_id).join("events.jsonl"));
    let decisions: Vec<&Value> = events
        .iter()
        .filter(|event| event["event"] == "result_recorded")
        .collect();
    assert_eq!(decisions.len(), 2);
    assert_eq!(decisions[0]["result"]["revision"], 1);
    assert_eq!(decisions[0]["result"]["merge_ready"], "fail");
    assert_eq!(decisions[1]["result"]["revision"], 2);
    assert_eq!(decisions[1]["result"]["merge_ready"], "pass");
}

#[test]
fn parallel_finish_calls_keep_unique_contiguous_revisions() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let mut children = Vec::new();
    for minutes in 1..=8 {
        children.push(
            harness
                .command()
                .args([
                    "measure",
                    "finish",
                    &run_id,
                    "--merge-ready",
                    "pass",
                    "--human-minutes",
                    &minutes.to_string(),
                ])
                .spawn()
                .unwrap(),
        );
    }
    for child in children {
        assert_success(&child.wait_with_output().unwrap());
    }

    let result = read_json(harness.run_path(&run_id).join("result.json"));
    assert_eq!(result["revision"], 8);
    let events = read_jsonl(harness.run_path(&run_id).join("events.jsonl"));
    let mut revisions: Vec<u64> = events
        .iter()
        .filter(|event| event["event"] == "result_recorded")
        .map(|event| event["result"]["revision"].as_u64().unwrap())
        .collect();
    revisions.sort_unstable();
    assert_eq!(revisions, (1..=8).collect::<Vec<_>>());
}
