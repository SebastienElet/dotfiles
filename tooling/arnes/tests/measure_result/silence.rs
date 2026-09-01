use super::measure_support::*;
use serde_json::{Value, json};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

const DAY_MS: u64 = 24 * 60 * 60 * 1_000;

#[test]
fn without_result_orders_oldest_observable_silence_first() {
    let harness = Harness::new();
    let now_ms = current_time_ms();
    let recent = harness.capture("codex", "session_id", "recent", "recent secret prompt");
    let old = harness.capture("claude-code", "session_id", "old", "old secret prompt");
    set_observed_times(&harness, &recent, now_ms - 2_000, now_ms - 1_000);
    set_observed_times(&harness, &old, now_ms - 4 * DAY_MS, now_ms - 3 * DAY_MS);

    let output = harness.run(&["measure", "list", "--without-result", "--format", "json"]);

    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let reported_at_ms = report["reported_at_ms"].as_u64().unwrap();
    let runs = report["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 2);
    assert_eq!(runs[0]["run_id"], old);
    assert_eq!(runs[0]["agent"], "claude-code");
    assert_eq!(runs[0]["started_at_ms"], now_ms - 4 * DAY_MS);
    assert_eq!(runs[0]["last_event"], "prompt.submit");
    assert_eq!(runs[0]["last_event_at_ms"], now_ms - 3 * DAY_MS);
    assert_eq!(runs[0]["start_to_last_event_ms"], DAY_MS);
    assert_eq!(
        runs[0]["silence_ms"],
        reported_at_ms - (now_ms - 3 * DAY_MS)
    );
    assert_eq!(runs[1]["run_id"], recent);
    assert_eq!(runs[1]["start_to_last_event_ms"], 1_000);
    assert_eq!(runs[1]["silence_ms"], reported_at_ms - (now_ms - 1_000));
    assert!(runs[0]["silence_ms"].as_u64() > runs[1]["silence_ms"].as_u64());
}

#[test]
fn without_result_marks_missing_and_incoherent_durations_unavailable() {
    let harness = Harness::new();
    let no_event = harness.capture("codex", "session_id", "no-event", "private prompt");
    fs::write(harness.run_path(&no_event).join("events.jsonl"), b"").unwrap();
    let future = harness.capture("codex", "session_id", "future", "private prompt");
    set_observed_times(&harness, &future, u64::MAX - 1, u64::MAX);
    let reversed = harness.capture("codex", "session_id", "reversed", "private prompt");
    set_observed_times(&harness, &reversed, 1_000, 2_000);
    append_event_at(&harness, &reversed, 1_500);

    let output = harness.run(&["measure", "list", "--without-result", "--format", "json"]);

    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(report["reported_at_ms"].as_u64().unwrap() > 0);
    for run_id in [&no_event, &future, &reversed] {
        let run = find_run(&report, run_id);
        assert_eq!(run["start_to_last_event_ms"], Value::Null);
        assert_eq!(run["silence_ms"], Value::Null);
    }
    let no_event_run = find_run(&report, &no_event);
    assert_eq!(no_event_run["last_event"], Value::Null);
    assert_eq!(no_event_run["last_event_at_ms"], Value::Null);
    let human =
        String::from_utf8(harness.run(&["measure", "list", "--without-result"]).stdout).unwrap();
    assert!(human.contains(&format!("{no_event} agent=codex started_at_ms=")));
    assert!(human.contains(
        "last_event=unavailable last_event_at_ms=unavailable start_to_last_event_ms=unavailable silence_ms=unavailable"
    ));
}

#[test]
fn without_result_excludes_every_run_with_structured_result_history() {
    let harness = Harness::new();
    let pending = harness.capture("codex", "session_id", "pending", "prompt");
    let recorded = harness.capture("codex", "session_id", "recorded", "prompt");
    let missing_snapshot = harness.capture("codex", "session_id", "missing", "prompt");
    finish(&harness, &recorded);
    finish(&harness, &missing_snapshot);
    fs::remove_file(harness.run_path(&missing_snapshot).join("result.json")).unwrap();

    let output = harness.run(&["measure", "list", "--without-result", "--format", "json"]);

    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    let runs = report["runs"].as_array().unwrap();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0]["run_id"], pending);
}

#[test]
fn without_result_human_and_json_outputs_exclude_sensitive_fields() {
    let harness = Harness::new();
    let prompt = "secret user content";
    let run_id = harness.capture("codex", "session_id", "session", prompt);
    let local_path = harness.repository.to_str().unwrap();

    let json_output = harness.run(&["measure", "list", "--without-result", "--format", "json"]);
    let human_output = harness.run(&["measure", "list", "--without-result"]);

    assert_success(&json_output);
    assert_success(&human_output);
    let json = String::from_utf8(json_output.stdout).unwrap();
    let human = String::from_utf8(human_output.stdout).unwrap();
    for output in [&json, &human] {
        assert!(output.contains(&run_id));
        assert!(output.contains("reported_at_ms"));
        assert!(output.contains("started_at_ms"));
        assert!(output.contains("last_event"));
        assert!(output.contains("last_event_at_ms"));
        assert!(output.contains("start_to_last_event_ms"));
        assert!(output.contains("silence_ms"));
        assert!(!output.contains(prompt));
        assert!(!output.contains(local_path));
        assert!(!output.contains("repository"));
    }
}

#[test]
fn without_result_keeps_invalid_event_records_visible_as_measurement_failures() {
    let harness = Harness::new();
    let run_id = harness.capture("codex", "session_id", "invalid", "private prompt");
    fs::write(
        harness.run_path(&run_id).join("events.jsonl"),
        b"{\"event\":\"prompt.submit\",\"timestamp_ms\":\"invalid\"}\n",
    )
    .unwrap();

    assert_failure(
        &harness.run(&["measure", "list", "--without-result"]),
        "events.jsonl has an invalid record",
    );
}

#[test]
fn without_result_help_limits_the_interpretation_of_silence() {
    let harness = Harness::new();

    let output = harness.run(&["measure", "list", "--help"]);

    assert_success(&output);
    let help = String::from_utf8(output.stdout).unwrap();
    assert!(help.contains("silence does not prove blockage, active time, or process state"));
}

fn current_time_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}

fn set_observed_times(harness: &Harness, run_id: &str, started_at_ms: u64, event_at_ms: u64) {
    let run_path = harness.run_path(run_id);
    let run_json = run_path.join("run.json");
    let mut run = read_json(&run_json);
    run["started_at_ms"] = json!(started_at_ms);
    fs::write(run_json, serde_json::to_vec(&run).unwrap()).unwrap();
    let events_path = run_path.join("events.jsonl");
    let mut events = read_jsonl(&events_path);
    events[0]["timestamp_ms"] = json!(event_at_ms);
    let content = events
        .into_iter()
        .map(|event| serde_json::to_string(&event).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(events_path, content).unwrap();
}

fn find_run<'a>(report: &'a Value, run_id: &str) -> &'a Value {
    report["runs"]
        .as_array()
        .unwrap()
        .iter()
        .find(|run| run["run_id"] == run_id)
        .unwrap()
}

fn append_event_at(harness: &Harness, run_id: &str, timestamp_ms: u64) {
    let path = harness.run_path(run_id).join("events.jsonl");
    let mut events = read_jsonl(&path);
    let mut event = events[0].clone();
    event["timestamp_ms"] = json!(timestamp_ms);
    event["event_id"] = json!("b".repeat(64));
    events.push(event);
    let content = events
        .iter()
        .map(|event| serde_json::to_string(event).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(path, content).unwrap();
}

fn finish(harness: &Harness, run_id: &str) {
    assert_success(&harness.run(&[
        "measure",
        "finish",
        run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ]));
}
