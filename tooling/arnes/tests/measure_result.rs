use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
    repository: PathBuf,
    state: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repository = root.path().join("repository");
        let state = root.path().join("state");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&repository).unwrap();
        fs::create_dir(&state).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(&repository)
                .status()
                .unwrap()
                .success()
        );
        Self {
            _root: root,
            home,
            repository,
            state,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arnes"));
        command
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn capture(&self, agent: &str, session_key: &str, session: &str, prompt: &str) -> String {
        let mut child = self
            .command()
            .args(["measure", "hook", "--agent", agent])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let payload = json!({
            session_key: session,
            "hook_event_name": "UserPromptSubmit",
            "prompt": prompt,
            "message_id": format!("message-{session}")
        });
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload.to_string().as_bytes())
            .unwrap();
        assert_success(&child.wait_with_output().unwrap());
        let runs = self.runs();
        let path = runs
            .iter()
            .find(|path| read_json(path.join("run.json"))["session_id"] == session)
            .unwrap();
        path.file_name().unwrap().to_str().unwrap().to_owned()
    }

    fn run(&self, arguments: &[&str]) -> Output {
        self.command().args(arguments).output().unwrap()
    }

    fn run_path(&self, run_id: &str) -> PathBuf {
        self.state.join("dotfiles/agent-harness/runs").join(run_id)
    }

    fn runs(&self) -> Vec<PathBuf> {
        fs::read_dir(self.state.join("dotfiles/agent-harness/runs"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    }
}

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
}

fn assert_failure(output: &Output, expected: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(expected), "stderr: {stderr}");
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn read_jsonl(path: impl AsRef<Path>) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

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
    fs::set_permissions(run.join("events.jsonl"), fs::Permissions::from_mode(0o400)).unwrap();

    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]);
    assert_failure(&output, "Permission denied");
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
