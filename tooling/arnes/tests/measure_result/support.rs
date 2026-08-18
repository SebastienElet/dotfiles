use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;

pub(super) struct Harness {
    _root: TempDir,
    home: PathBuf,
    pub(super) repository: PathBuf,
    state: PathBuf,
}

impl Harness {
    pub(super) fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repository = root.path().join("repository");
        let state = root.path().join("state");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&repository).unwrap();
        fs::create_dir(&state).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(&repository)
                .status()
                .unwrap()
                .success()
        );
        Self {
            _root: root,
            home,
            repository,
            state,
        }
    }

    pub(super) fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arnes"));
        command
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    pub(super) fn capture(
        &self,
        agent: &str,
        session_key: &str,
        session: &str,
        prompt: &str,
    ) -> String {
        let mut child = self
            .command()
            .args(["measure", "hook", "--agent", agent])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        let payload = json!({
            session_key: session,
            "hook_event_name": "UserPromptSubmit",
            "prompt": prompt,
            "message_id": format!("message-{session}")
        });
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload.to_string().as_bytes())
            .unwrap();
        assert_success(&child.wait_with_output().unwrap());
        let runs = self.runs();
        let path = runs
            .iter()
            .find(|path| read_json(path.join("run.json"))["session_id"] == session)
            .unwrap();
        path.file_name().unwrap().to_str().unwrap().to_owned()
    }

    pub(super) fn run(&self, arguments: &[&str]) -> Output {
        self.command().args(arguments).output().unwrap()
    }

    pub(super) fn run_path(&self, run_id: &str) -> PathBuf {
        self.state.join("dotfiles/agent-harness/runs").join(run_id)
    }

    pub(super) fn runs(&self) -> Vec<PathBuf> {
        fs::read_dir(self.state.join("dotfiles/agent-harness/runs"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    }
}

pub(super) fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
}

pub(super) fn assert_failure(output: &Output, expected: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(expected), "stderr: {stderr}");
}

pub(super) fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

pub(super) fn read_jsonl(path: impl AsRef<Path>) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}
