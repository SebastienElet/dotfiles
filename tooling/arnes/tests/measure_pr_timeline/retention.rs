use super::support::*;
use std::io::Write;
use std::os::unix::fs::symlink;
use std::time::{SystemTime, UNIX_EPOCH};

const DAY_MS: u64 = 86_400_000;

#[test]
fn expires_the_whole_pr_timeline_at_90_days() {
    for age in [89, 90, 91] {
        let harness = Harness::new();
        assert_status(&harness.record(&[]), "recorded");
        let timeline = harness.timeline();
        age_events(&harness, age);
        force_sweep(&harness);
        assert!(trigger_hook(&harness).stderr.is_empty());
        assert_eq!(timeline.parent().unwrap().exists(), age < 90);
        let state = retention_state(&harness);
        assert_eq!(state["status"], "complete");
        assert_eq!(state["removed_prs"], u64::from(age >= 90));
    }
}

#[test]
fn a_new_head_preserves_the_entire_history_and_triggers_maintenance() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    age_events(&harness, 91);
    force_sweep(&harness);
    assert_status(&harness.record(&[("--head-sha", OTHER_SHA)]), "recorded");
    assert_eq!(harness.events().len(), 2);
    assert_eq!(retention_state(&harness)["removed_prs"], 0);
}

#[test]
fn sweeps_prs_at_most_once_per_day() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    age_events(&harness, 91);
    assert!(trigger_hook(&harness).stderr.is_empty());
    assert_eq!(harness.events().len(), 1);
    force_sweep(&harness);
    let path = harness.timeline();
    assert!(trigger_hook(&harness).stderr.is_empty());
    assert!(!path.exists());
}

#[test]
fn a_duplicate_does_not_refresh_the_expiration_clock() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    age_events(&harness, 91);
    let path = harness.timeline();
    force_sweep(&harness);
    assert_status(&harness.record(&[]), "duplicate");
    assert!(!path.exists());
    assert_status(&harness.record(&[]), "recorded");
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn refuses_unsafe_or_unproven_expiration_without_deleting_data() {
    for damage in ["future", "empty", "truncated", "identity", "symlink"] {
        let harness = Harness::new();
        assert_status(&harness.record(&[]), "recorded");
        age_events(&harness, 91);
        let timeline = harness.timeline();
        let mut event = harness.events().remove(0);
        match damage {
            "future" => {
                event["timestamp_ms"] = json!(now_ms() + DAY_MS);
                write_events(&harness, &[event]);
            }
            "empty" => fs::write(&timeline, "").unwrap(),
            "truncated" => fs::write(&timeline, "{").unwrap(),
            "identity" => {
                event["pr"]["pr_id"] = json!(999);
                write_events(&harness, &[event]);
            }
            _ => {
                let outside = harness.root.path().join("outside");
                fs::write(&outside, "sentinel").unwrap();
                symlink(outside, timeline.parent().unwrap().join("unsafe")).unwrap();
            }
        }
        let before = fs::read(&timeline).unwrap();
        force_sweep(&harness);
        let output = trigger_hook(&harness);
        assert_eq!(output.status.code(), Some(0));
        assert!(!output.stderr.is_empty(), "{damage}");
        assert_eq!(retention_state(&harness)["status"], "failed");
        assert_eq!(fs::read(timeline).unwrap(), before);
    }
}

#[test]
fn concurrent_writers_and_purge_preserve_every_new_head() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    age_events(&harness, 91);
    force_sweep(&harness);
    let children: Vec<_> = (0..16)
        .map(|index| {
            let sha = format!("{index:040x}");
            harness.command(&[("--head-sha", &sha)]).spawn().unwrap()
        })
        .collect();
    let hook = trigger_hook(&harness);
    assert!(
        hook.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&hook.stderr)
    );
    for child in children {
        assert_status(&child.wait_with_output().unwrap(), "recorded");
    }
    let events = harness.events();
    for index in 0..16 {
        assert_eq!(
            events
                .iter()
                .filter(|event| event["head_sha"] == format!("{index:040x}"))
                .count(),
            1
        );
    }
}

#[test]
fn reads_the_existing_retention_state_before_upgrading_it() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    let path = harness.measure_root().join("retention.json");
    fs::write(&path, json!({"schema_version":1,"status":"complete","swept_at_ms":1,"next_sweep_at_ms":2,"candidate_runs":0,"removed_runs":0}).to_string()).unwrap();
    assert!(trigger_hook(&harness).stderr.is_empty());
    assert_eq!(retention_state(&harness)["schema_version"], 2);
}

#[test]
fn invalid_maintenance_state_reports_failure_after_preserving_the_new_verdict() {
    for invalid in [
        json!({}),
        json!({"schema_version":2,"status":"complete","swept_at_ms":1,"next_sweep_at_ms":2,"candidate_runs":0,"removed_runs":0}),
        json!({"schema_version":2,"status":"complete","swept_at_ms":1,"next_sweep_at_ms":2,"candidate_runs":0,"removed_runs":0,"candidate_prs":0,"removed_prs":1}),
    ] {
        let harness = Harness::new();
        assert_status(&harness.record(&[]), "recorded");
        fs::write(
            harness.measure_root().join("retention.json"),
            invalid.to_string(),
        )
        .unwrap();
        assert_error(
            &harness.record(&[("--head-sha", OTHER_SHA)]),
            "PR event recorded; retention failed:",
        );
        assert_eq!(harness.events().len(), 2);
        assert_eq!(harness.events()[1]["data"]["verdict"], "changes-required");
        assert_eq!(retention_state(&harness), invalid);
    }
}

fn trigger_hook(harness: &Harness) -> Output {
    let mut child = harness
        .base_command()
        .args(["measure", "hook", "--agent", "codex"])
        .stdin(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"retention-trigger","hook_event_name":"Stop"}"#)
        .unwrap();
    child.wait_with_output().unwrap()
}

fn age_events(harness: &Harness, days: u64) {
    let mut events = harness.events();
    for event in &mut events {
        event["timestamp_ms"] = json!(now_ms() - days * DAY_MS);
    }
    write_events(harness, &events);
}

fn write_events(harness: &Harness, events: &[Value]) {
    fs::write(
        harness.timeline(),
        events
            .iter()
            .map(|event| format!("{event}\n"))
            .collect::<String>(),
    )
    .unwrap();
}

fn force_sweep(harness: &Harness) {
    let path = harness.measure_root().join("retention.json");
    if path.exists() {
        fs::remove_file(path).unwrap();
    }
}

fn retention_state(harness: &Harness) -> Value {
    serde_json::from_slice(&fs::read(harness.measure_root().join("retention.json")).unwrap())
        .unwrap()
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .try_into()
        .unwrap()
}
