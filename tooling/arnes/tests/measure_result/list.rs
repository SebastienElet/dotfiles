use super::measure_support::*;
use serde_json::{Value, json};
use std::fs;

#[test]
fn lists_pending_runs_with_repository_prompt_event_and_agent_filter() {
    let harness = Harness::new();
    let codex = harness.capture(
        "codex",
        "session_id",
        "codex-session",
        "  first\n prompt with enough detail  ",
    );
    harness.capture(
        "cursor",
        "conversation_id",
        "cursor-session",
        "cursor prompt",
    );

    let output = harness.run(&["measure", "list", "--agent", "codex", "--format", "json"]);
    assert_success(&output);
    let runs: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(runs.as_array().unwrap().len(), 1);
    assert_eq!(runs[0]["run_id"], codex);
    assert_eq!(runs[0]["agent"], "codex");
    assert_eq!(
        runs[0]["repository"],
        fs::canonicalize(&harness.repository)
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(
        runs[0]["first_prompt_excerpt"],
        "first prompt with enough detail"
    );
    assert_eq!(runs[0]["last_event"], "prompt.submit");
    assert_eq!(runs[0]["has_result"], false);
}

#[test]
fn list_rejects_prompt_records_missing_contract_fields() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    fs::write(
        harness.run_path(&run_id).join("prompts.jsonl"),
        b"{\"prompt\":\"fake prompt\"}\n",
    )
    .unwrap();

    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"]),
        "prompts.jsonl has an invalid record",
    );
}

#[test]
fn list_rejects_duplicate_prompt_fields() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    fs::write(
        harness.run_path(&run_id).join("prompts.jsonl"),
        b"{\"timestamp_ms\":1,\"event_id\":\"event\",\"session_id\":\"session\",\"prompt_id\":null,\"prompt\":\"first\",\"prompt\":\"second\"}\n",
    )
    .unwrap();

    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"]),
        "prompts.jsonl has an invalid record",
    );
}

#[test]
fn list_rejects_event_records_with_wrong_field_types() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    fs::write(
        harness.run_path(&run_id).join("events.jsonl"),
        b"{\"event\":\"prompt.submit\",\"timestamp_ms\":\"invalid\"}\n",
    )
    .unwrap();

    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"]),
        "events.jsonl has an invalid record",
    );
}

#[test]
fn list_rejects_duplicate_event_fields() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let path = harness.run_path(&run_id).join("events.jsonl");
    let content = fs::read_to_string(&path).unwrap().replace(
        "\"event\":\"prompt.submit\"",
        "\"event\":\"prompt.submit\",\"event\":\"agent.stop\"",
    );
    fs::write(path, content).unwrap();

    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"]),
        "events.jsonl has an invalid record",
    );
}

#[test]
fn list_rejects_an_event_log_without_a_terminal_newline() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let path = harness.run_path(&run_id).join("events.jsonl");
    let mut content = fs::read(&path).unwrap();
    assert_eq!(content.pop(), Some(b'\n'));
    fs::write(path, content).unwrap();

    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"]),
        "events.jsonl is truncated or oversized",
    );
}

#[test]
fn human_list_marks_a_finalized_run_without_inference_from_agent_stop() {
    let harness = Harness::new();
    let run_id = harness.capture("claude-code", "session_id", "session", "implement it");
    let before = harness.run(&["measure", "list"]);
    assert_success(&before);
    let before = String::from_utf8(before.stdout).unwrap();
    assert!(before.contains(&run_id));
    assert!(before.contains("result=pending"));

    let finish = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "2.5",
    ]);
    assert_success(&finish);
    let after = String::from_utf8(harness.run(&["measure", "list"]).stdout).unwrap();
    assert!(after.contains("result=recorded"));
}

#[test]
fn list_reports_a_missing_result_snapshot_without_repairing_it() {
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
    let events_before = fs::read(run.join("events.jsonl")).unwrap();
    fs::remove_file(run.join("result.json")).unwrap();

    let output = harness.run(&["measure", "list", "--format", "json"]);

    assert_success(&output);
    let runs: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(runs[0]["has_result"], false);
    assert_eq!(runs[0]["result_state"], "missing");
    assert!(!run.join("result.json").exists());
    assert_eq!(fs::read(run.join("events.jsonl")).unwrap(), events_before);
}

#[test]
fn list_reports_a_one_revision_lag_without_repairing_it() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "session", "prompt");
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]));
    let revision_one = fs::read(&result_path).unwrap();
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "2",
    ]));
    fs::write(&result_path, revision_one).unwrap();
    let result_before = fs::read(&result_path).unwrap();
    let events_before = fs::read(run.join("events.jsonl")).unwrap();

    let output = harness.run(&["measure", "list", "--format", "json"]);

    assert_success(&output);
    let runs: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(runs[0]["has_result"], true);
    assert_eq!(runs[0]["result_state"], "lagging");
    assert_eq!(fs::read(result_path).unwrap(), result_before);
    assert_eq!(fs::read(run.join("events.jsonl")).unwrap(), events_before);
}

#[test]
fn human_list_escapes_terminal_controls_without_changing_json() {
    let harness = Harness::new();
    let prompt = "prompt\u{1b}]52;c;dGVzdA==\u{7}\u{9d}\u{7f}";
    let repository = "/repo/\u{1b}[31m\u{9d}\u{7f}";
    let run_id = harness.capture("codex", "session_id", "session", prompt);
    let run_path = harness.run_path(&run_id);
    let run_json = run_path.join("run.json");
    let mut run = read_json(&run_json);
    run["repository"] = json!(repository);
    fs::write(&run_json, serde_json::to_vec(&run).unwrap()).unwrap();

    let human = harness.run(&["measure", "list"]);

    assert_success(&human);
    let human = String::from_utf8(human.stdout).unwrap();
    assert!(
        human
            .chars()
            .all(|character| character == '\n' || !character.is_control())
    );
    assert!(human.contains("\\u{001b}]52"));
    assert!(human.contains("\\u{009d}"));
    assert!(human.contains("\\u{007f}"));
    let json_output = harness.run(&["measure", "list", "--format", "json"]);
    assert_success(&json_output);
    let runs: Value = serde_json::from_slice(&json_output.stdout).unwrap();
    assert_eq!(runs[0]["repository"], repository);
    assert_eq!(runs[0]["first_prompt_excerpt"], prompt);
}
