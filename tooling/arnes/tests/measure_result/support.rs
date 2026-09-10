use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;
pub struct Harness {
    _root: TempDir,
    home: PathBuf,
    pub(super) repository: PathBuf,
    state: PathBuf,
    legacy_runs: bool,
}
impl Harness {
    pub(super) fn new_v2() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_legacy_runs(false)
    }
    pub(super) fn with_legacy_runs(
        legacy_runs: bool,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let home = root.path().join("home");
        let repository = root.path().join("repository");
        let state = root.path().join("state");
        fs::create_dir(&home)?;
        fs::create_dir(&repository)?;
        fs::create_dir(&state)?;
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .current_dir(&repository)
                .status()?
                .success()
        );
        Ok(Self {
            _root: root,
            home,
            repository,
            state,
            legacy_runs,
        })
    }
    pub(super) fn command(&self) -> std::process::Command {
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
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let before = self.runs()?;
        let mut child = self
            .command()
            .args(["measure", "hook", "--agent", agent])
            .stdin(Stdio::piped())
            .spawn()?;
        let payload = json ! ({ session_key : session , "hook_event_name" : "UserPromptSubmit" , "prompt" : prompt , "message_id" : format ! ("message-{session}") });
        child
            .stdin
            .take()
            .ok_or("required test value is missing")?
            .write_all(payload.to_string().as_bytes())?;
        assert_success(&child.wait_with_output()?);
        let path = self
            .runs()?
            .into_iter()
            .find(|path| !before.contains(path))
            .ok_or("required test value is missing")?;
        let run_id = path
            .file_name()
            .ok_or("required test value is missing")?
            .to_str()
            .ok_or("required test value is missing")?
            .to_owned();
        if self.legacy_runs {
            self.convert_to_v1(&run_id, session)?;
        }
        Ok(run_id)
    }
    pub(super) fn run(
        &self,
        arguments: &[&str],
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.command().args(arguments).output()?)
    }
    pub(super) fn run_path(&self, run_id: &str) -> PathBuf {
        self.state_root().join("runs").join(run_id)
    }
    pub(super) fn state_root(&self) -> PathBuf {
        self.state.join("dotfiles/agent-harness")
    }
    pub(super) fn runs(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let runs = self.state.join("dotfiles/agent-harness/runs");
        if !runs.exists() {
            return Ok(Vec::new());
        }
        fs::read_dir(runs)?
            .map(
                |entry| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(entry?.path())
                },
            )
            .collect::<Result<Vec<_>, _>>()
    }
    fn convert_to_v1(
        &self,
        run_id: &str,
        session: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = self.run_path(run_id).join("run.json");
        let mut run = read_json(&path)?;
        {
            run.as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(("schema_version").to_owned(), json!(1));
        };
        {
            run.as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(("session_id").to_owned(), json!(session));
        };
        {
            run.as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(("repository").to_owned(), Value::Null);
        };
        {
            run.as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(("repository_branch").to_owned(), Value::Null);
        };
        {
            run.as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(("model").to_owned(), Value::Null);
        };
        run.as_object_mut()
            .ok_or("expected JSON object")?
            .remove("model_fingerprint");
        run.as_object_mut()
            .ok_or("expected JSON object")?
            .remove("operating_system");
        run.as_object_mut()
            .ok_or("expected JSON object")?
            .remove("architecture");
        fs::write(path, serde_json::to_vec(&run)?)?;
        Ok(())
    }
}
pub fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty());
}
pub fn assert_failure(output: &Output, expected: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(expected), "stderr: {stderr}");
}
pub fn read_json(
    path: impl AsRef<Path>,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
pub fn read_jsonl(
    path: impl AsRef<Path>,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    fs::read_to_string(path)?
        .lines()
        .map(
            |line| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(serde_json::from_str(line)?)
            },
        )
        .collect::<Result<Vec<_>, _>>()
}
