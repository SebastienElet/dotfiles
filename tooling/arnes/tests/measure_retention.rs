#[path = "measure_result/support.rs"]
mod measure_support;

use measure_support::*;
use serde_json::json;
use std::fs;
use std::os::unix::fs::symlink;
use std::time::{SystemTime, UNIX_EPOCH};

const DAY_MS: u64 = 86_400_000;

#[test]
fn expires_v2_after_60_days_and_never_automatically_removes_v1() {
    let harness = Harness::new_v2();
    let before_boundary = harness.capture("codex", "session_id", "before", "fixture prompt");
    let at_boundary = harness.capture("codex", "session_id", "boundary", "fixture prompt");
    let after_boundary = harness.capture("codex", "session_id", "after", "fixture prompt");
    let legacy = harness.capture("codex", "session_id", "legacy", "fixture prompt");
    let reference = now_ms();
    set_run_age(&harness, &before_boundary, reference - 60 * DAY_MS + 60_000);
    set_run_age(&harness, &at_boundary, reference - 60 * DAY_MS);
    set_run_age(&harness, &after_boundary, reference - 61 * DAY_MS);
    age_run(&harness, &legacy, 61);
    convert_run_to_v1(&harness, &legacy);
    force_sweep(&harness);

    harness.capture("codex", "session_id", "trigger", "fixture prompt");

    assert!(harness.run_path(&before_boundary).is_dir());
    assert!(!harness.run_path(&at_boundary).exists());
    assert!(!harness.run_path(&after_boundary).exists());
    assert!(harness.run_path(&legacy).is_dir());
    assert_failure(
        &harness.run(&[
            "measure",
            "outcome",
            &at_boundary,
            "--status",
            "pass",
            "--oracle",
            "cargo-test",
        ]),
        "unknown run",
    );
    let state = read_json(harness.state_root().join("retention.json"));
    assert_eq!(state["schema_version"], 1);
    assert_eq!(state["status"], "complete");
    assert_eq!(state["candidate_runs"], 2);
    assert_eq!(state["removed_runs"], 2);
}

#[test]
fn preserves_a_run_when_observed_timestamps_are_incoherent() {
    let harness = Harness::new_v2();
    let run_id = harness.capture("codex", "session_id", "incoherent", "fixture prompt");
    let path = harness.run_path(&run_id).join("events.jsonl");
    let mut events = read_jsonl(&path);
    let mut older = events[0].clone();
    older["timestamp_ms"] = json!(now_ms() - 61 * DAY_MS);
    events.push(older);
    write_events(&path, &events);
    force_sweep(&harness);

    let output = harness.hook(
        "codex",
        json!({"session_id":"trigger","hook_event_name":"SessionStart"}),
    );

    assert_eq!(output.status.code(), Some(0));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("retention refuses incoherent event timestamps")
    );
    assert!(harness.run_path(&run_id).is_dir());
    assert_eq!(harness.runs().len(), 2);
    assert_eq!(
        read_json(harness.state_root().join("retention.json"))["status"],
        "failed"
    );
}

#[test]
fn suppresses_another_sweep_for_one_day() {
    let harness = Harness::new_v2();
    harness.capture("codex", "session_id", "first", "fixture prompt");
    force_sweep(&harness);
    harness.capture("codex", "session_id", "sweep", "fixture prompt");
    let expired = harness.capture("codex", "session_id", "expired", "fixture prompt");
    age_run(&harness, &expired, 61);

    harness.capture("codex", "session_id", "trigger", "fixture prompt");

    assert!(harness.run_path(&expired).is_dir());
}

#[test]
fn a_fresh_outcome_postpones_expiration() {
    let harness = Harness::new_v2();
    let run_id = harness.capture("codex", "session_id", "judged", "fixture prompt");
    age_run(&harness, &run_id, 61);
    assert_success(&harness.run(&[
        "measure",
        "outcome",
        &run_id,
        "--status",
        "pass",
        "--oracle",
        "cargo-test",
    ]));
    force_sweep(&harness);

    harness.capture("codex", "session_id", "trigger", "fixture prompt");

    assert!(harness.run_path(&run_id).is_dir());
}

#[test]
fn refuses_unsafe_expired_entries_without_partial_deletion() {
    let harness = Harness::new_v2();
    let first = harness.capture("codex", "session_id", "first-expired", "fixture prompt");
    let second = harness.capture("codex", "session_id", "second-expired", "fixture prompt");
    let mut run_ids = [first, second];
    run_ids.sort();
    let safe = &run_ids[0];
    let unsafe_run = &run_ids[1];
    age_run(&harness, safe, 61);
    age_run(&harness, unsafe_run, 61);
    let outside = harness.repository.join("outside");
    fs::write(&outside, "sentinel").unwrap();
    symlink(&outside, harness.run_path(unsafe_run).join("unsafe")).unwrap();
    force_sweep(&harness);

    let output = harness.hook(
        "codex",
        json!({"session_id":"trigger","hook_event_name":"SessionStart"}),
    );

    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("measure hook:"), "{stderr}");
    assert!(harness.run_path(safe).is_dir());
    assert!(harness.run_path(unsafe_run).is_dir());
    assert_eq!(harness.runs().len(), 3);
    assert_eq!(fs::read_to_string(outside).unwrap(), "sentinel");
    assert_eq!(
        read_json(harness.state_root().join("retention.json"))["status"],
        "failed"
    );
}

fn age_run(harness: &Harness, run_id: &str, days: u64) {
    set_run_age(harness, run_id, now_ms() - days * DAY_MS);
}

fn set_run_age(harness: &Harness, run_id: &str, timestamp_ms: u64) {
    let run = harness.run_path(run_id);
    let metadata_path = run.join("run.json");
    let mut metadata = read_json(&metadata_path);
    metadata["started_at_ms"] = json!(timestamp_ms);
    fs::write(metadata_path, serde_json::to_vec(&metadata).unwrap()).unwrap();
    let path = run.join("events.jsonl");
    let mut events = read_jsonl(&path);
    events.last_mut().unwrap()["timestamp_ms"] = json!(timestamp_ms);
    write_events(&path, &events);
}

fn write_events(path: &std::path::Path, events: &[serde_json::Value]) {
    let content = events
        .iter()
        .map(|event| serde_json::to_string(event).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(path, content).unwrap();
}

fn convert_run_to_v1(harness: &Harness, run_id: &str) {
    let path = harness.run_path(run_id).join("run.json");
    let current = read_json(&path);
    let legacy = json!({
        "schema_version":1,
        "run_id":run_id,
        "agent":"codex",
        "session_id":"legacy-session",
        "started_at_ms":current["started_at_ms"],
        "model":null,
        "repository":null,
        "repository_commit":current["repository_commit"],
        "repository_branch":null,
        "repository_dirty":current["repository_dirty"],
        "harness_fingerprint":current["harness_fingerprint"],
        "harness_fingerprint_limitations":current["harness_fingerprint_limitations"]
    });
    fs::write(path, serde_json::to_vec_pretty(&legacy).unwrap()).unwrap();
}

fn force_sweep(harness: &Harness) {
    let path = harness.state_root().join("retention.json");
    if path.exists() {
        fs::remove_file(path).unwrap();
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}
