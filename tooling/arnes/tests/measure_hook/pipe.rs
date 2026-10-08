use super::support::*;

#[test]
fn closed_stdin_preserves_the_application_refusal_diagnostic()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let mut command = harness.command("codex");
    command.env("XDG_STATE_HOME", "relative/state");
    let mut child = command.spawn()?;
    let stdin = child.stdin.take().ok_or("missing child stdin")?;
    assert!(child.wait()?.success());
    child.stdin = Some(stdin);
    let output = collect_hook_output(child, &vec![b'x'; 131_072])?;
    assert_advisory_failure(&output);
    assert!(String::from_utf8(output.stderr)?.contains("absolute"));
    assert!(!harness.measure_root().exists());
    Ok(())
}

#[test]
fn closed_stdin_preserves_a_nonzero_child_status_and_stderr()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let mut child = harness.command("codex").arg("--unknown-option").spawn()?;
    let stdin = child.stdin.take().ok_or("missing child stdin")?;
    assert_eq!(child.wait()?.code(), Some(2));
    child.stdin = Some(stdin);
    let output = collect_hook_output(child, &vec![b'x'; 131_072])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("--unknown-option"));
    assert!(!harness.measure_root().exists());
    Ok(())
}
