use agent_handoff::{Environment, HandoffError, run_agent_handoff};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

#[path = "cli/runtime_parity.rs"]
mod runtime_parity;

struct Fixture {
    _root: TempDir,
    home: PathBuf,
    state: PathBuf,
    transcript: PathBuf,
}

struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("write failed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Fixture {
    fn new() -> Self {
        let root = TempDir::new().unwrap();
        Self {
            home: root.path().join("home"),
            state: root.path().join("state"),
            transcript: root.path().join("transcript.jsonl"),
            _root: root,
        }
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_agent-handoff"));
        command
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .env("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "100000");
        command
    }

    fn sentinel(&self, session_id: &str) -> PathBuf {
        self.state.join("dotfiles/handoff").join(session_id)
    }

    fn write_claude_usage(&self, used: u64) {
        let record = r#"{"type":"assistant","isSidechain":false,"message":{"usage":{"cache_creation_input_tokens":0,"cache_read_input_tokens":0,"input_tokens":USED}}}"#
            .replace("USED", &used.to_string());
        fs::write(&self.transcript, format!("{record}\n")).unwrap();
    }
}

fn event(transcript: &Path, session_id: &str, stop_hook_active: bool) -> Vec<u8> {
    format!(
        "{{\"hook_event_name\":\"Stop\",\"session_id\":\"{session_id}\",\"stop_hook_active\":{stop_hook_active},\"transcript_path\":\"{}\"}}",
        transcript.display()
    )
    .into_bytes()
}

fn run(mut command: Command, input: &[u8]) -> Output {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}

fn run_event(fixture: &Fixture, session_id: &str) -> Output {
    run(
        fixture.command(),
        &event(&fixture.transcript, session_id, false),
    )
}

fn assert_clean_success(output: &Output) {
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"");
}

#[test]
fn below_threshold_usage_exits_cleanly_without_creating_a_sentinel() {
    let fixture = Fixture::new();
    fixture.write_claude_usage(84_999);

    let output = run_event(&fixture, "below");

    assert_clean_success(&output);
    assert!(!fixture.sentinel("below").exists());
}

#[test]
fn threshold_usage_writes_exact_block_bytes_and_creates_the_sentinel() {
    let fixture = Fixture::new();
    fixture.write_claude_usage(85_000);

    let output = run_event(&fixture, "block");

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        output.stdout,
        b"{\n  \"decision\": \"block\",\n  \"reason\": \"Context is at 85k tokens, past the 85k handoff threshold. Start no new work. Use /handoff to emit the resume prompt for a fresh session, then stop.\"\n}\n"
    );
    assert_eq!(output.stderr, b"");
    assert!(fixture.sentinel("block").is_file());
}

#[test]
fn output_write_errors_are_classified_as_unexpected_after_sentinel_creation() {
    let fixture = Fixture::new();
    fixture.write_claude_usage(85_000);
    let environment = Environment {
        claude_code_auto_compact_window: Some("100000".into()),
        xdg_state_home: Some(fixture.state.to_string_lossy().into_owned()),
        home: Some(fixture.home.to_string_lossy().into_owned()),
        ..Environment::default()
    };

    let error = run_agent_handoff(
        &event(&fixture.transcript, "write-error", false),
        &environment,
        &mut FailingWriter,
    )
    .unwrap_err();

    assert_eq!(error, HandoffError::unexpected("unexpected failure"));
    assert!(fixture.sentinel("write-error").is_file());
}

#[test]
fn transcript_read_errors_use_the_usage_exit_contract_without_a_sentinel() {
    let fixture = Fixture::new();

    let output = run_event(&fixture, "missing");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"agent-handoff: cannot read transcript\n");
    assert!(!fixture.sentinel("missing").exists());
}

#[test]
fn invalid_transcripts_do_not_create_a_sentinel() {
    let fixture = Fixture::new();
    fs::write(&fixture.transcript, b"not-json\n").unwrap();

    let output = run_event(&fixture, "invalid");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"agent-handoff: malformed transcript JSON at retained line 1\n"
    );
    assert!(!fixture.sentinel("invalid").exists());
}

#[test]
fn invalid_thresholds_do_not_create_a_sentinel() {
    let fixture = Fixture::new();
    fixture.write_claude_usage(90_000);
    let mut command = fixture.command();
    command.env("HANDOFF_TOKEN_THRESHOLD", "invalid");

    let output = run(command, &event(&fixture.transcript, "threshold", false));

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"agent-handoff: invalid HANDOFF_TOKEN_THRESHOLD\n"
    );
    assert!(!fixture.sentinel("threshold").exists());
}

#[test]
fn invalid_sentinel_types_use_the_unexpected_exit_contract() {
    let fixture = Fixture::new();
    let sentinel = fixture.sentinel("directory");
    fs::create_dir_all(&sentinel).unwrap();

    let output = run_event(&fixture, "directory");

    assert_eq!(output.status.code(), Some(3));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"agent-handoff: cannot inspect handoff sentinel\n"
    );
}

#[test]
fn recursive_stop_returns_before_environment_and_transcript_access() {
    let fixture = Fixture::new();
    let mut command = Command::new(env!("CARGO_BIN_EXE_agent-handoff"));
    command.env_clear();

    let output = run(command, &event(&fixture.transcript, "recursive", true));

    assert_clean_success(&output);
}

#[test]
fn an_existing_sentinel_returns_before_transcript_access() {
    let fixture = Fixture::new();
    let sentinel = fixture.sentinel("existing");
    fs::create_dir_all(sentinel.parent().unwrap()).unwrap();
    File::create(&sentinel).unwrap();

    let output = run_event(&fixture, "existing");

    assert_clean_success(&output);
}

#[test]
fn extra_cli_arguments_are_ignored() {
    let fixture = Fixture::new();
    fixture.write_claude_usage(84_999);
    let mut command = fixture.command();
    command.arg("ignored");

    let output = run(
        command,
        &event(&fixture.transcript, "extra-argument", false),
    );

    assert_clean_success(&output);
    assert!(!fixture.sentinel("extra-argument").exists());
}
