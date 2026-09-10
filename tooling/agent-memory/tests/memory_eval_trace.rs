#![cfg(test)]

use serde_json::Value;
use std::fs;
use std::net::Shutdown;
use std::os::fd::OwnedFd;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

struct TraceFixture {
    _temporary: tempfile::TempDir,
    repository: PathBuf,
    root: PathBuf,
}

impl TraceFixture {
    fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temporary = tempfile::tempdir()?;
        let repository = temporary.path().join("repository");
        fs::create_dir(&repository)?;
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repository)
            .status()?;
        assert!(status.success());
        let root = temporary.path().join("store");
        Ok(Self {
            _temporary: temporary,
            repository,
            root,
        })
    }
}

#[test]
fn absent_trace_environment_has_no_trace_io() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = TraceFixture::new()?;
    let trace_root = tempfile::tempdir()?;

    let output = run(&fixture, ["audit", "--format", "json"], b"", None)?;

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fs::read_dir(trace_root.path())?.count(), 0);
    Ok(())
}

#[test]
fn writes_private_minimal_start_and_completion_events()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = TraceFixture::new()?;
    let trace_root = tempfile::tempdir()?;
    let trace = trace_root.path().join("session.jsonl");

    let output = run(
        &fixture,
        ["audit", "--format", "json"],
        b"",
        Some((trace_root.path(), &trace, "cursor")),
    )?;

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fs::metadata(&trace)?.permissions().mode() & 0o777, 0o600);
    let events = trace_events(&trace)?;
    assert_eq!(events.len(), 2);
    assert_event(
        events.first().ok_or("missing fixture element")?,
        "started",
        "cursor",
        "audit",
        "started",
    )?;
    assert_event(
        events.get(1).ok_or("missing fixture element")?,
        "completed",
        "cursor",
        "audit",
        "success",
    )?;
    Ok(())
}

#[test]
fn writes_a_redacted_error_event_before_failure_output()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = TraceFixture::new()?;
    let trace_root = tempfile::tempdir()?;
    let trace = trace_root.path().join("error.jsonl");
    let secret = b"ghp_trace_secret_value";

    let output = run(
        &fixture,
        ["admit", "--format", "json"],
        secret,
        Some((trace_root.path(), &trace, "codex")),
    )?;

    assert_eq!(output.status.code(), Some(2));
    let bytes = fs::read(&trace)?;
    assert!(!bytes.windows(secret.len()).any(|window| window == secret));
    let events = trace_events(&trace)?;
    assert_event(
        events.first().ok_or("missing fixture element")?,
        "started",
        "codex",
        "admit",
        "started",
    )?;
    assert_event(
        events.get(1).ok_or("missing fixture element")?,
        "error",
        "codex",
        "admit",
        "rejection",
    )?;
    Ok(())
}

#[test]
fn refuses_existing_symlink_relative_and_outside_trace_paths()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = TraceFixture::new()?;
    let trace_root = tempfile::tempdir()?;
    let outside = tempfile::tempdir()?;
    let existing = trace_root.path().join("existing.jsonl");
    fs::write(&existing, b"sentinel")?;
    let linked = trace_root.path().join("linked.jsonl");
    symlink(&existing, &linked)?;

    for trace in [
        existing.clone(),
        linked,
        outside.path().join("outside.jsonl"),
        PathBuf::from("relative.jsonl"),
    ] {
        let output = run(
            &fixture,
            ["audit", "--format", "json"],
            b"",
            Some((trace_root.path(), &trace, "claude")),
        )?;
        assert_eq!(output.status.code(), Some(4), "trace={}", trace.display());
    }
    assert_eq!(fs::read(&existing)?, b"sentinel");

    let linked_root = outside.path().join("linked-root");
    symlink(trace_root.path(), &linked_root)?;
    let output = run(
        &fixture,
        ["audit", "--format", "json"],
        b"",
        Some((&linked_root, &linked_root.join("root.jsonl"), "claude")),
    )?;
    assert_eq!(output.status.code(), Some(4));

    let real_parent = trace_root.path().join("real-parent/nested");
    fs::create_dir_all(&real_parent)?;
    let linked_parent = trace_root.path().join("linked-parent");
    symlink(trace_root.path().join("real-parent"), &linked_parent)?;
    let intermediate = linked_parent.join("nested/intermediate.jsonl");
    let output = run(
        &fixture,
        ["audit", "--format", "json"],
        b"",
        Some((trace_root.path(), &intermediate, "claude")),
    )?;
    assert_eq!(output.status.code(), Some(4));
    Ok(())
}

#[test]
fn records_output_failure_instead_of_success()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = TraceFixture::new()?;
    let trace_root = tempfile::tempdir()?;
    let trace = trace_root.path().join("broken-output.jsonl");
    let (output, peer) = UnixStream::pair()?;
    peer.shutdown(Shutdown::Read)?;
    drop(peer);
    let status = traced_command(
        &fixture,
        ["audit", "--format", "json"],
        Some((trace_root.path(), &trace, "codex")),
    )
    .stdout(Stdio::from(OwnedFd::from(output)))
    .status()?;
    assert_eq!(status.code(), Some(4));
    let events = trace_events(&trace)?;
    assert_event(
        events.get(1).ok_or("missing fixture element")?,
        "error",
        "codex",
        "audit",
        "unavailable",
    )?;
    Ok(())
}

fn run<const N: usize>(
    fixture: &TraceFixture,
    arguments: [&str; N],
    input: &[u8],
    trace: Option<(&Path, &Path, &str)>,
) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
    let mut command = traced_command(fixture, arguments, trace);
    let mut child = command.spawn()?;
    std::io::Write::write_all(
        child.stdin.as_mut().ok_or("child stdin is not piped")?,
        input,
    )?;
    Ok(child.wait_with_output()?)
}

fn traced_command<const N: usize>(
    fixture: &TraceFixture,
    arguments: [&str; N],
    trace: Option<(&Path, &Path, &str)>,
) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_agent-memory"));
    command
        .args(arguments)
        .current_dir(&fixture.repository)
        .env("AGENT_MEMORY_ROOT", &fixture.root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if let Some((root, path, agent)) = trace {
        command
            .env("AGENT_MEMORY_EVAL_ROOT", root)
            .env("AGENT_MEMORY_EVAL_TRACE", path)
            .env("AGENT_MEMORY_EVAL_AGENT", agent);
    }
    command
}

fn trace_events(path: &Path) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    let input = fs::read_to_string(path)?;
    Ok(input
        .lines()
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?)
}

fn assert_event(
    event: &Value,
    name: &str,
    agent: &str,
    command: &str,
    exit_class: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let object = event
        .as_object()
        .ok_or("trace event must be a JSON object")?;
    let mut keys = object.keys().map(String::as_str).collect::<Vec<_>>();
    keys.sort_unstable();
    assert_eq!(
        keys,
        [
            "agent",
            "command",
            "event",
            "exit_class",
            "pid",
            "timestamp_ms"
        ]
    );
    assert_eq!(
        object
            .get("event")
            .ok_or("trace event field event missing")?,
        name
    );
    assert_eq!(
        object
            .get("agent")
            .ok_or("trace event field agent missing")?,
        agent
    );
    assert_eq!(
        object
            .get("command")
            .ok_or("trace event field command missing")?,
        command
    );
    assert_eq!(
        object
            .get("exit_class")
            .ok_or("trace event field exit_class missing")?,
        exit_class
    );
    assert!(object.get("pid").and_then(Value::as_u64).is_some());
    assert!(object.get("timestamp_ms").and_then(Value::as_u64).is_some());
    Ok(())
}
