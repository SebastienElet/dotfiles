use serde_json::Value;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

struct TraceFixture {
    _temporary: tempfile::TempDir,
    repository: PathBuf,
    root: PathBuf,
}

impl TraceFixture {
    fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let repository = temporary.path().join("repository");
        fs::create_dir(&repository).unwrap();
        let status = Command::new("git")
            .args(["init", "-q"])
            .current_dir(&repository)
            .status()
            .unwrap();
        assert!(status.success());
        let root = temporary.path().join("store");
        Self {
            _temporary: temporary,
            repository,
            root,
        }
    }
}

#[test]
fn absent_trace_environment_has_no_trace_io() {
    let fixture = TraceFixture::new();
    let trace_root = tempfile::tempdir().unwrap();

    let output = run(&fixture, ["audit", "--format", "json"], b"", None);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fs::read_dir(trace_root.path()).unwrap().count(), 0);
}

#[test]
fn writes_private_minimal_start_and_completion_events() {
    let fixture = TraceFixture::new();
    let trace_root = tempfile::tempdir().unwrap();
    let trace = trace_root.path().join("session.jsonl");

    let output = run(
        &fixture,
        ["audit", "--format", "json"],
        b"",
        Some((trace_root.path(), &trace, "cursor")),
    );

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        fs::metadata(&trace).unwrap().permissions().mode() & 0o777,
        0o600
    );
    let events = trace_events(&trace);
    assert_eq!(events.len(), 2);
    assert_event(&events[0], "started", "cursor", "audit", "started");
    assert_event(&events[1], "completed", "cursor", "audit", "success");
}

#[test]
fn writes_a_redacted_error_event_before_failure_output() {
    let fixture = TraceFixture::new();
    let trace_root = tempfile::tempdir().unwrap();
    let trace = trace_root.path().join("error.jsonl");
    let secret = b"ghp_trace_secret_value";

    let output = run(
        &fixture,
        ["admit", "--format", "json"],
        secret,
        Some((trace_root.path(), &trace, "codex")),
    );

    assert_eq!(output.status.code(), Some(2));
    let bytes = fs::read(&trace).unwrap();
    assert!(!bytes.windows(secret.len()).any(|window| window == secret));
    let events = trace_events(&trace);
    assert_event(&events[0], "started", "codex", "admit", "started");
    assert_event(&events[1], "error", "codex", "admit", "rejection");
}

#[test]
fn refuses_existing_symlink_relative_and_outside_trace_paths() {
    let fixture = TraceFixture::new();
    let trace_root = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    let existing = trace_root.path().join("existing.jsonl");
    fs::write(&existing, b"sentinel").unwrap();
    let linked = trace_root.path().join("linked.jsonl");
    symlink(&existing, &linked).unwrap();

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
        );
        assert_eq!(output.status.code(), Some(4), "trace={}", trace.display());
    }
    assert_eq!(fs::read(&existing).unwrap(), b"sentinel");

    let linked_root = outside.path().join("linked-root");
    symlink(trace_root.path(), &linked_root).unwrap();
    let output = run(
        &fixture,
        ["audit", "--format", "json"],
        b"",
        Some((&linked_root, &linked_root.join("root.jsonl"), "claude")),
    );
    assert_eq!(output.status.code(), Some(4));

    let real_parent = trace_root.path().join("real-parent/nested");
    fs::create_dir_all(&real_parent).unwrap();
    let linked_parent = trace_root.path().join("linked-parent");
    symlink(trace_root.path().join("real-parent"), &linked_parent).unwrap();
    let intermediate = linked_parent.join("nested/intermediate.jsonl");
    let output = run(
        &fixture,
        ["audit", "--format", "json"],
        b"",
        Some((trace_root.path(), &intermediate, "claude")),
    );
    assert_eq!(output.status.code(), Some(4));
}

#[test]
fn records_output_failure_instead_of_success() {
    let fixture = TraceFixture::new();
    let trace_root = tempfile::tempdir().unwrap();
    let trace = trace_root.path().join("broken-output.jsonl");
    let mut child = traced_command(
        &fixture,
        ["audit", "--format", "json"],
        Some((trace_root.path(), &trace, "codex")),
    )
    .spawn()
    .unwrap();
    drop(child.stdout.take());
    let status = child.wait().unwrap();
    assert_eq!(status.code(), Some(4));
    let events = trace_events(&trace);
    assert_event(&events[1], "error", "codex", "audit", "unavailable");
}

fn run<const N: usize>(
    fixture: &TraceFixture,
    arguments: [&str; N],
    input: &[u8],
    trace: Option<(&Path, &Path, &str)>,
) -> Output {
    let mut command = traced_command(fixture, arguments, trace);
    let mut child = command.spawn().unwrap();
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), input).unwrap();
    child.wait_with_output().unwrap()
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

fn trace_events(path: &Path) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

fn assert_event(event: &Value, name: &str, agent: &str, command: &str, exit_class: &str) {
    let object = event.as_object().unwrap();
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
    assert_eq!(event["event"], name);
    assert_eq!(event["agent"], agent);
    assert_eq!(event["command"], command);
    assert_eq!(event["exit_class"], exit_class);
    assert!(event["pid"].as_u64().is_some());
    assert!(event["timestamp_ms"].as_u64().is_some());
}
