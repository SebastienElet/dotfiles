pub(super) use serde_json::{Value, json};
pub(super) use std::fs;
use std::path::PathBuf;
pub(super) use std::process::{Command, Output, Stdio};

pub(super) const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub(super) const OTHER_SHA: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

pub(super) struct Harness {
    pub root: tempfile::TempDir,
}

impl Harness {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::create_dir(root.path().join("repository")).unwrap();
        fs::create_dir(root.path().join("home")).unwrap();
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(root.path().join("repository"))
                .status()
                .unwrap()
                .success()
        );
        Self { root }
    }

    pub fn base_command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arnes"));
        command
            .current_dir(self.root.path().join("repository"))
            .env_clear()
            .env("HOME", self.root.path().join("home"))
            .env("XDG_STATE_HOME", self.root.path().join("state"))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    pub fn command(&self, overrides: &[(&str, &str)]) -> Command {
        let mut command = self.base_command();
        command.args(["measure", "pr-verdict"]);
        for (flag, default) in [
            ("--forge", "github.com"),
            ("--repository", "example/project"),
            ("--pr-id", "1042"),
            ("--head-sha", SHA),
            ("--agent", "codex"),
            ("--verdict", "changes-required"),
            ("--blocking-findings", "2"),
            ("--non-blocking-findings", "1"),
            ("--observable-behaviors", "3"),
            ("--evidence-gaps", "2"),
        ] {
            let value = overrides
                .iter()
                .find(|(key, _)| *key == flag)
                .map_or(default, |(_, value)| *value);
            command.args([flag, value]);
        }
        command
    }

    pub fn record(&self, overrides: &[(&str, &str)]) -> Output {
        self.command(overrides).output().unwrap()
    }

    pub fn measure_root(&self) -> PathBuf {
        self.root.path().join("state/dotfiles/agent-harness")
    }

    pub fn timeline(&self) -> PathBuf {
        let entries: Vec<_> = fs::read_dir(self.measure_root().join("pull-requests"))
            .unwrap()
            .collect();
        assert_eq!(entries.len(), 1);
        entries[0].as_ref().unwrap().path().join("events.jsonl")
    }

    pub fn events(&self) -> Vec<Value> {
        fs::read_to_string(self.timeline())
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }
}

pub(super) fn assert_status(output: &Output, expected: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
    assert!(output.stderr.is_empty());
}

pub(super) fn assert_error(output: &Output, expected: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
