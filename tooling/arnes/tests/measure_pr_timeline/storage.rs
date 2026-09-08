use super::support::*;
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn output_failure_preserves_the_recorded_event_and_allows_an_idempotent_retry() {
    let harness = Harness::new();
    let destination = harness.root.path().join("read-only-stdout");
    fs::write(&destination, "untouched").unwrap();
    let output = harness
        .command(&[])
        .stdout(fs::File::open(&destination).unwrap())
        .output()
        .unwrap();
    assert_error(&output, "measure:");
    assert_eq!(fs::read(destination).unwrap(), b"untouched");
    assert_eq!(harness.events().len(), 1);
    assert_status(&harness.record(&[]), "duplicate");
    assert_eq!(harness.events().len(), 1);
}

#[test]
fn storage_failure_is_separate_from_the_review_and_can_be_retried() {
    let harness = Harness::new();
    let state = harness.root.path().join("state");
    fs::write(&state, "occupied").unwrap();
    assert_error(&harness.record(&[]), "measure:");
    assert_eq!(fs::read(&state).unwrap(), b"occupied");
    fs::remove_file(state).unwrap();
    assert_status(&harness.record(&[]), "recorded");
    assert_eq!(harness.events()[0]["data"]["verdict"], "changes-required");
}

#[test]
fn refuses_invalid_anchors_and_counts_without_writing_events() {
    for (flag, value) in [
        ("--forge", "https://github.com"),
        ("--forge", "../escape"),
        ("--repository", "../escape"),
        ("--repository", "/local/path"),
        ("--repository", "example/project.git"),
        ("--repository", "example//project"),
        ("--pr-id", "0"),
        ("--head-sha", "abcdef"),
        ("--head-sha", "zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz"),
        ("--blocking-findings", "-1"),
        ("--observable-behaviors", "unknown"),
        ("--verdict", "looks good"),
        ("--agent", "unknown"),
    ] {
        let harness = Harness::new();
        assert_error(&harness.record(&[(flag, value)]), "");
        assert!(!harness.measure_root().join("pull-requests").exists());
    }
}

#[test]
fn refuses_a_corrupt_or_conflicting_history_without_appending() {
    for damage in ["truncated", "version", "identity", "duplicate"] {
        let harness = Harness::new();
        assert_status(&harness.record(&[]), "recorded");
        let mut event = harness.events().remove(0);
        let content = match damage {
            "truncated" => "{\"schema_version\":".to_owned(),
            "version" => {
                event["schema_version"] = json!(99);
                format!("{event}\n")
            }
            "identity" => {
                event["pr"]["pr_id"] = json!(999);
                format!("{event}\n")
            }
            _ => format!("{event}\n{event}\n"),
        };
        fs::write(harness.timeline(), &content).unwrap();
        assert_error(&harness.record(&[("--head-sha", OTHER_SHA)]), "measure:");
        assert_eq!(fs::read_to_string(harness.timeline()).unwrap(), content);
    }
}

#[test]
fn preserves_existing_measurement_runs_and_outcomes() {
    let harness = Harness::new();
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
        .write_all(br#"{"session_id":"fixture","hook_event_name":"Stop"}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = fs::read_dir(harness.measure_root().join("runs"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let outcome = harness
        .base_command()
        .args([
            "measure",
            "outcome",
            run.file_name().unwrap().to_str().unwrap(),
            "--status",
            "pass",
            "--oracle",
            "fixture-test",
        ])
        .output()
        .unwrap();
    assert!(outcome.status.success());
    let before: Vec<_> = ["run.json", "events.jsonl", "outcomes.jsonl"]
        .map(|name| (name, fs::read(run.join(name)).unwrap()))
        .into();
    assert_status(&harness.record(&[]), "recorded");
    for (name, bytes) in before {
        assert_eq!(fs::read(run.join(name)).unwrap(), bytes);
    }
    for args in [
        vec!["measure", "list", "--format", "json"],
        vec!["measure", "report", "--format", "json"],
    ] {
        let output = harness.base_command().args(args).output().unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<Value>(&output.stdout).unwrap();
    }
}

#[test]
fn keeps_events_private_and_refuses_symlink_redirection() {
    let harness = Harness::new();
    assert_status(&harness.record(&[]), "recorded");
    assert_eq!(
        fs::metadata(harness.timeline())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o600
    );
    let outside = harness.root.path().join("outside");
    fs::write(&outside, "untouched").unwrap();
    fs::remove_file(harness.timeline()).unwrap();
    symlink(&outside, harness.timeline()).unwrap();
    assert_error(&harness.record(&[]), "measure:");
    assert_eq!(fs::read(outside).unwrap(), b"untouched");
}

#[test]
fn refuses_state_inside_the_checkout() {
    let harness = Harness::new();
    let repository = harness.root.path().join("repository");
    let output = harness
        .command(&[])
        .env("XDG_STATE_HOME", repository.join("state"))
        .output()
        .unwrap();
    assert_error(&output, "inside the repository");
    assert!(!repository.join("state").exists());
}
