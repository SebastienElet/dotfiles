pub(super) use serde_json::{Value, json};
pub(super) use std::fs;
pub(super) use std::io::{Seek, SeekFrom, Write};
pub(super) use std::os::unix::fs::{PermissionsExt, symlink};
pub(super) use std::path::{Path, PathBuf};
pub(super) use std::process::{Child, Command, Output, Stdio};
pub(super) use tempfile::TempDir;

pub(super) struct Harness {
    pub(super) _root: TempDir,
    pub(super) home: PathBuf,
    pub(super) repository: PathBuf,
    pub(super) state: PathBuf,
}

impl Harness {
    pub(super) fn new() -> Self {
        Self::with_repository_name("repository")
    }

    pub(super) fn with_repository_name(name: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repository = root.path().join(name);
        let state = root.path().join("state");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&repository).unwrap();
        fs::create_dir(&state).unwrap();
        Self {
            _root: root,
            home,
            repository,
            state,
        }
    }

    pub(super) fn run(&self, agent: &str, payload: &[u8]) -> Output {
        let mut child = self.command(agent).spawn().unwrap();
        child.stdin.take().unwrap().write_all(payload).unwrap();
        child.wait_with_output().unwrap()
    }

    pub(super) fn command(&self, agent: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arnes"));
        command
            .args(["measure", "hook", "--agent", agent])
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    pub(super) fn list(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["measure", "list", "--format", "json"])
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .output()
            .unwrap()
    }

    pub(super) fn measure_root(&self) -> PathBuf {
        self.state.join("dotfiles/agent-harness")
    }

    pub(super) fn runs(&self) -> Vec<PathBuf> {
        let root = self.measure_root().join("runs");
        if !root.exists() {
            return Vec::new();
        }
        fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    }

    pub(super) fn only_run(&self) -> PathBuf {
        let runs = self.runs();
        assert_eq!(runs.len(), 1, "expected one run, found {runs:?}");
        runs[0].clone()
    }
}

pub(super) fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

pub(super) fn assert_advisory_failure(output: &Output) {
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

pub(super) fn run_at(harness: &Harness, current: &Path, state: &Path, payload: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["measure", "hook", "--agent", "codex"])
        .current_dir(current)
        .env_clear()
        .env("HOME", &harness.home)
        .env("XDG_STATE_HOME", state)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();
    child.wait_with_output().unwrap()
}

pub(super) fn run_record(harness: &Harness, session: &str) -> Value {
    harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == session)
        .unwrap()
}

pub(super) fn capture_run(
    harness: &Harness,
    agent: &str,
    session_key: &str,
    session: &str,
) -> Value {
    let mut payload = json!({});
    payload[session_key] = json!(session);
    assert_success(&harness.run(agent, payload.to_string().as_bytes()));
    run_record(harness, session)
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

pub(super) fn walk(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![root.to_owned()];
    let mut index = 0;
    while let Some(path) = paths.get(index).cloned() {
        index += 1;
        if path.is_dir() {
            paths.extend(
                fs::read_dir(path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        }
    }
    paths
}

pub(super) fn init_repository(repository: &Path, branch: &str, tracked: &str) {
    git(repository, &["init", "-b", branch]);
    fs::write(repository.join(tracked), tracked).unwrap();
    git(repository, &["add", tracked]);
    commit(repository, "initial");
}

pub(super) fn git(repository: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

pub(super) fn commit(repository: &Path, message: &str) {
    git(
        repository,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            message,
        ],
    );
}

pub(super) fn git_value(repository: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
