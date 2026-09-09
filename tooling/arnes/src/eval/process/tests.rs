use super::*;

fn shell(script: &str, stdin: &str, timeout: Duration) -> CaptureResult {
    capture(
        Path::new("/bin/sh"),
        &["-c".into(), script.into()],
        CaptureOptions {
            cwd: Path::new("/tmp"),
            env: &BTreeMap::new(),
            stdin,
            timeout,
        },
    )
}

#[test]
fn forwards_exact_utf8_stdin() {
    let result = shell("/bin/cat", " é\n\n", Duration::from_secs(2));
    assert_eq!(result.error, None);
    assert_eq!(result.output, " é\n\n");
}

#[test]
fn contains_nonzero_exit_timeout_output_overflow_and_missing_executable() {
    assert_eq!(
        shell("exit 9", "", Duration::from_secs(2)).error,
        Some(ExecutionError::AgentFailed)
    );
    assert_eq!(
        shell("/bin/sleep 10", "", Duration::from_millis(100)).error,
        Some(ExecutionError::Timeout)
    );
    assert_eq!(
        shell("/usr/bin/yes x", "", Duration::from_secs(2)).error,
        Some(ExecutionError::OutputLimit)
    );
    let result = capture(
        Path::new("/nonexistent-provider"),
        &[],
        CaptureOptions {
            cwd: Path::new("/tmp"),
            env: &BTreeMap::new(),
            stdin: "",
            timeout: Duration::from_secs(1),
        },
    );
    assert_eq!(result.error, Some(ExecutionError::AgentFailed));
}

#[test]
fn times_out_and_kills_descendants_that_hold_output_open() {
    let started = std::time::Instant::now();
    let result = shell("/bin/sleep 10 & exit 9", "", Duration::from_millis(150));
    assert!(started.elapsed() < Duration::from_secs(2));
    assert_eq!(result.error, Some(ExecutionError::Timeout));
}

#[test]
fn bounds_stdin_writes_when_child_does_not_read() {
    let started = std::time::Instant::now();
    let result = shell(
        "/bin/sleep 10",
        &"é".repeat(2_000_000),
        Duration::from_millis(100),
    );
    assert_eq!(result.error, Some(ExecutionError::Timeout));
    assert!(started.elapsed() < Duration::from_secs(2));
}
