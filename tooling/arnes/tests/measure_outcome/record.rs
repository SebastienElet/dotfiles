use super::measure_support::*;
use std::fs;

#[test]
fn help_exposes_the_outcome_contract_and_safe_replacement() {
    let harness = Harness::new_v2();
    let output = harness.run(&["measure", "outcome", "--help"]);

    assert_success(&output);
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("Record an explicit outcome backed by a named oracle"));
    assert!(help.contains("Append a replacement revision"));
    assert!(help.contains("does not infer success"));
}

#[test]
fn records_an_oracle_backed_outcome_and_replay_is_a_no_op() {
    let harness = Harness::new_v2();
    let run_id = harness.capture("codex", "session_id", "session", "fixture prompt");
    assert_eq!(
        read_json(harness.run_path(&run_id).join("run.json"))["schema_version"],
        2
    );
    let arguments = [
        "measure",
        "outcome",
        &run_id,
        "--status",
        "pass",
        "--oracle",
        "cargo-test",
    ];

    assert_success(&harness.run(&arguments));
    let path = harness.run_path(&run_id).join("outcomes.jsonl");
    let before = fs::read(&path).unwrap();
    assert_success(&harness.run(&arguments));

    assert_eq!(fs::read(&path).unwrap(), before);
    let outcomes = read_jsonl(path);
    assert_eq!(outcomes.len(), 1);
    assert_eq!(outcomes[0]["revision"], 1);
    assert_eq!(outcomes[0]["status"], "pass");
    assert_eq!(outcomes[0]["oracle"], "cargo-test");
    assert!(outcomes[0]["reason"].is_null());
    let listed = harness.run(&["measure", "list", "--format", "json"]);
    assert_success(&listed);
    let listed: serde_json::Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed[0]["result_state"], "outcome-recorded");
    let pending = harness.run(&["measure", "list", "--without-result", "--format", "json"]);
    assert_success(&pending);
    let pending: serde_json::Value = serde_json::from_slice(&pending.stdout).unwrap();
    assert!(pending["runs"].as_array().unwrap().is_empty());
}

#[test]
fn conflicting_outcome_requires_explicit_replacement() {
    let harness = Harness::new_v2();
    let run_id = harness.capture("claude-code", "session_id", "session", "fixture prompt");
    assert_success(&harness.run(&[
        "measure",
        "outcome",
        &run_id,
        "--status",
        "pass",
        "--oracle",
        "cargo-test",
    ]));

    let conflicting = [
        "measure",
        "outcome",
        &run_id,
        "--status",
        "fail",
        "--oracle",
        "cargo-test",
    ];
    assert_failure(
        &harness.run(&conflicting),
        "outcome already differs; use --replace",
    );
    let mut replacement = conflicting.to_vec();
    replacement.push("--replace");
    assert_success(&harness.run(&replacement));

    let outcomes = read_jsonl(harness.run_path(&run_id).join("outcomes.jsonl"));
    assert_eq!(outcomes.len(), 2);
    assert_eq!(outcomes[1]["revision"], 2);
    assert_eq!(outcomes[1]["status"], "fail");
}
