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
    legacy_runs: bool,
}

impl Harness {
    #[allow(dead_code)]
    pub(super) fn new() -> Self {
        Self::with_legacy_runs(true)
    }

    pub(super) fn new_v2() -> Self {
        Self::with_legacy_runs(false)
    }

    fn with_legacy_runs(legacy_runs: bool) -> Self {
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
            legacy_runs,
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
        let before = self.runs();
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
        let path = self
            .runs()
            .into_iter()
            .find(|path| !before.contains(path))
            .unwrap();
        let run_id = path.file_name().unwrap().to_str().unwrap().to_owned();
        if self.legacy_runs {
            self.convert_to_v1(&run_id, session);
        }
        run_id
    }

    pub(super) fn run(&self, arguments: &[&str]) -> Output {
        self.command().args(arguments).output().unwrap()
    }

    #[allow(dead_code)]
    pub(super) fn hook(&self, agent: &str, payload: Value) -> Output {
        let mut child = self
            .command()
            .args(["measure", "hook", "--agent", agent])
            .stdin(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(payload.to_string().as_bytes())
            .unwrap();
        child.wait_with_output().unwrap()
    }

    pub(super) fn run_path(&self, run_id: &str) -> PathBuf {
        self.state.join("dotfiles/agent-harness/runs").join(run_id)
    }

    #[allow(dead_code)]
    pub(super) fn state_root(&self) -> PathBuf {
        self.state.join("dotfiles/agent-harness")
    }

    pub(super) fn runs(&self) -> Vec<PathBuf> {
        let runs = self.state.join("dotfiles/agent-harness/runs");
        if !runs.exists() {
            return Vec::new();
        }
        fs::read_dir(runs)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    }

    fn convert_to_v1(&self, run_id: &str, session: &str) {
        let path = self.run_path(run_id).join("run.json");
        let mut run = read_json(&path);
        run["schema_version"] = json!(1);
        run["session_id"] = json!(session);
        run["repository"] = Value::Null;
        run["repository_branch"] = Value::Null;
        run["model"] = Value::Null;
        run.as_object_mut().unwrap().remove("model_fingerprint");
        run.as_object_mut().unwrap().remove("operating_system");
        run.as_object_mut().unwrap().remove("architecture");
        fs::write(path, serde_json::to_vec(&run).unwrap()).unwrap();
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
