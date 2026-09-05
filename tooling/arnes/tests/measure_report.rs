#[path = "measure_result/support.rs"]
mod measure_support;

use measure_support::*;
use serde_json::{Value, json};
use std::fs;

#[test]
fn reports_judgeability_success_activity_latency_and_volume_by_agent() {
    let harness = Harness::new_v2();
    let passing = harness.capture("codex", "session_id", "passing", "fixture prompt");
    assert_success(&harness.hook(
        "codex",
        json!({"session_id":"passing","hook_event_name":"PreToolUse"}),
    ));
    let failing = harness.capture("claude-code", "session_id", "failing", "fixture prompt");
    let unjudgeable = harness.capture("cursor", "conversation_id", "unjudgeable", "fixture prompt");
    harness.capture("codex", "session_id", "pending", "fixture prompt");
    set_times(&harness, &passing, 100, &[200, 300]);
    set_times(&harness, &failing, 400, &[500]);
    set_times(&harness, &unjudgeable, 600, &[700]);
    record_outcome(&harness, &passing, "pass", &["--oracle", "cargo-test"]);
    record_outcome(&harness, &failing, "fail", &["--oracle", "cargo-test"]);
    record_outcome(
        &harness,
        &unjudgeable,
        "unjudgeable",
        &["--reason", "missing-oracle"],
    );

    let output = harness.run(&["measure", "report", "--format", "json"]);
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert!(harness.state_root().is_dir());
    assert_eq!(report["totals"]["runs"], 4);
    assert_eq!(report["totals"]["judgeable_runs"], 2);
    assert_eq!(report["totals"]["judgeable_rate"], 0.5);
    assert_eq!(report["totals"]["declared_successful_runs"], 1);
    assert_eq!(report["totals"]["declared_success_rate"], 0.5);
    assert_eq!(report["totals"]["event_count"], 5);
    assert_eq!(report["totals"]["tool_call_count"], 1);
    assert_eq!(report["totals"]["latency_runs"], 4);
    assert!(report["storage"]["logical_bytes"].as_u64().unwrap() > 0);
    assert!(report["storage"]["allocated_bytes"].as_u64().unwrap() > 0);
    assert_eq!(report["agents"][0]["agent"], "codex");
    assert_eq!(report["agents"][0]["metrics"]["runs"], 2);
    assert_eq!(report["agents"][1]["agent"], "claude-code");
    assert_eq!(report["agents"][2]["agent"], "cursor");
}

#[test]
fn rejects_invalid_event_data_instead_of_omitting_the_run() {
    let harness = Harness::new_v2();
    let run_id = harness.capture("codex", "session_id", "invalid", "fixture prompt");
    fs::write(
        harness.run_path(&run_id).join("events.jsonl"),
        b"{\"schema_version\":2,\"timestamp_ms\":\"invalid\"}\n",
    )
    .unwrap();

    assert_failure(
        &harness.run(&["measure", "report", "--format", "json"]),
        "events.jsonl has an invalid record",
    );
}

#[test]
fn reports_unavailable_rates_as_null_and_never_exposes_private_context() {
    let harness = Harness::new_v2();
    let private = "fixture-private-context";
    harness.capture("codex", "session_id", "pending", private);

    let output = harness.run(&["measure", "report", "--format", "json"]);
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();

    assert_eq!(report["totals"]["judgeable_rate"], 0.0);
    assert!(report["totals"]["declared_success_rate"].is_null());
    assert_eq!(report["agents"].as_array().unwrap().len(), 1);
    assert_eq!(report["agents"][0]["agent"], "codex");
    assert!(!String::from_utf8(output.stdout).unwrap().contains(private));
}

fn record_outcome(harness: &Harness, run_id: &str, status: &str, fields: &[&str]) {
    let mut arguments = vec!["measure", "outcome", run_id, "--status", status];
    arguments.extend_from_slice(fields);
    assert_success(&harness.run(&arguments));
}

fn set_times(harness: &Harness, run_id: &str, started_at_ms: u64, events: &[u64]) {
    let run = harness.run_path(run_id);
    let mut metadata = read_json(run.join("run.json"));
    metadata["started_at_ms"] = json!(started_at_ms);
    fs::write(
        run.join("run.json"),
        serde_json::to_vec_pretty(&metadata).unwrap(),
    )
    .unwrap();
    let mut records = read_jsonl(run.join("events.jsonl"));
    for (record, timestamp) in records.iter_mut().zip(events) {
        record["timestamp_ms"] = json!(timestamp);
    }
    let bytes = records
        .iter()
        .map(|record| serde_json::to_string(record).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(run.join("events.jsonl"), bytes).unwrap();
}
