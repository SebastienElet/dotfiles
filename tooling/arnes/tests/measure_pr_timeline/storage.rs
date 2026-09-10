use super::support::*;
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, symlink};
#[test]
fn output_failure_preserves_the_recorded_event_and_allows_an_idempotent_retry()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let destination = harness.root.path().join("read-only-stdout");
    fs::write(&destination, "untouched")?;
    let output = harness
        .command(&[])
        .stdout(fs::File::open(&destination)?)
        .output()?;
    assert_error(&output, "measure:");
    assert_eq!(fs::read(destination)?, b"untouched");
    assert_eq!(harness.events()?.len(), 1);
    assert_status(&harness.record(&[])?, "duplicate");
    assert_eq!(harness.events()?.len(), 1);
    Ok(())
}
#[test]
fn storage_failure_is_separate_from_the_review_and_can_be_retried()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let state = harness.root.path().join("state");
    fs::write(&state, "occupied")?;
    assert_error(&harness.record(&[])?, "measure:");
    assert_eq!(fs::read(&state)?, b"occupied");
    fs::remove_file(state)?;
    assert_status(&harness.record(&[])?, "recorded");
    assert_eq!(
        *(*(*(harness.events()?)
            .first()
            .ok_or("missing fixture index 0")?)
        .get("data")
        .ok_or("missing fixture index data")?)
        .get("verdict")
        .ok_or("missing fixture index verdict")?,
        "changes-required"
    );
    Ok(())
}
#[test]
fn refuses_invalid_anchors_and_counts_without_writing_events()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (flag, value) in [
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
        let harness = Harness::new()?;
        assert_error(&harness.record(&[(flag, value)])?, "");
        assert!(!harness.measure_root().join("pull-requests").exists());
    };
    Ok(())
}
#[test]
fn refuses_a_corrupt_or_conflicting_history_without_appending()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for damage in ["truncated", "version", "identity", "duplicate"] {
        let harness = Harness::new()?;
        assert_status(&harness.record(&[])?, "recorded");
        let mut event = harness.events()?.remove(0);
        let content = match damage {
            "truncated" => "{\"schema_version\":".to_owned(),
            "version" => {
                {
                    event
                        .as_object_mut()
                        .ok_or("expected JSON object for fixture update")?
                        .insert(("schema_version").to_owned(), json!(99));
                };
                format!("{event}\n")
            }
            "identity" => {
                {
                    (event)
                        .get_mut("pr")
                        .ok_or("missing fixture index pr")?
                        .as_object_mut()
                        .ok_or("expected JSON object for fixture update")?
                        .insert(("pr_id").to_owned(), json!(999));
                };
                format!("{event}\n")
            }
            _ => format!("{event}\n{event}\n"),
        };
        fs::write(harness.timeline()?, &content)?;
        assert_error(&harness.record(&[("--head-sha", OTHER_SHA)])?, "measure:");
        assert_eq!(fs::read_to_string(harness.timeline()?)?, content);
    };
    Ok(())
}
#[test]
fn preserves_existing_measurement_runs_and_outcomes()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let mut child = harness
        .base_command()
        .args(["measure", "hook", "--agent", "codex"])
        .stdin(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(br#"{"session_id":"fixture","hook_event_name":"Stop"}"#)?;
    let output = child.wait_with_output()?;
    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let run = fs::read_dir(harness.measure_root().join("runs"))?
        .next()
        .ok_or("required test value is missing")??
        .path();
    let outcome = harness
        .base_command()
        .args([
            "measure",
            "outcome",
            run.file_name()
                .ok_or("required test value is missing")?
                .to_str()
                .ok_or("required test value is missing")?,
            "--status",
            "pass",
            "--oracle",
            "fixture-test",
        ])
        .output()?;
    assert!(outcome.status.success());
    let before: Vec<_> = ["run.json", "events.jsonl", "outcomes.jsonl"]
        .map(
            |name| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok((name, fs::read(run.join(name))?))
            },
        )
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
    assert_status(&harness.record(&[])?, "recorded");
    for (name, bytes) in before {
        assert_eq!(fs::read(run.join(name))?, bytes);
    }
    let _: () = for args in [
        vec!["measure", "list", "--format", "json"],
        vec!["measure", "report", "--format", "json"],
    ] {
        let output = harness.base_command().args(args).output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<Value>(&output.stdout)?;
    };
    Ok(())
}
#[test]
fn keeps_events_private_and_refuses_symlink_redirection()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_status(&harness.record(&[])?, "recorded");
    assert_eq!(
        fs::metadata(harness.timeline()?)?.permissions().mode() & 0o777,
        0o600
    );
    let outside = harness.root.path().join("outside");
    fs::write(&outside, "untouched")?;
    fs::remove_file(harness.timeline()?)?;
    symlink(&outside, harness.timeline()?)?;
    assert_error(&harness.record(&[])?, "measure:");
    assert_eq!(fs::read(outside)?, b"untouched");
    Ok(())
}
#[test]
fn refuses_state_inside_the_checkout() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let repository = harness.root.path().join("repository");
    let output = harness
        .command(&[])
        .env("XDG_STATE_HOME", repository.join("state"))
        .output()?;
    assert_error(&output, "inside the repository");
    assert!(!repository.join("state").exists());
    Ok(())
}
