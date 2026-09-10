pub use serde_json::{Value, json};
pub use std::fs;
use std::path::PathBuf;
pub use std::process::{Command, Output, Stdio};
pub const SHA: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
pub const OTHER_SHA: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
pub struct Harness {
    pub root: tempfile::TempDir,
}
impl Harness {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        fs::create_dir(root.path().join("repository"))?;
        fs::create_dir(root.path().join("home"))?;
        assert!(
            Command::new("git")
                .args(["init", "-q"])
                .current_dir(root.path().join("repository"))
                .status()?
                .success()
        );
        Ok(Self { root })
    }
    pub fn base_command(&self) -> std::process::Command {
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
    pub fn command(&self, overrides: &[(&str, &str)]) -> std::process::Command {
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
    pub fn record(
        &self,
        overrides: &[(&str, &str)],
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.command(overrides).output()?)
    }
    pub fn measure_root(&self) -> PathBuf {
        self.root.path().join("state/dotfiles/agent-harness")
    }
    pub fn timeline(&self) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let entries = fs::read_dir(self.measure_root().join("pull-requests"))?
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(entries.len(), 1);
        Ok(entries
            .first()
            .ok_or("expected a timeline directory")?
            .path()
            .join("events.jsonl"))
    }
    pub fn events(&self) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
        fs::read_to_string(self.timeline()?)?
            .lines()
            .map(
                |line| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(serde_json::from_str(line)?)
                },
            )
            .collect::<Result<Vec<_>, _>>()
    }
}
pub fn assert_status(output: &Output, expected: &str) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), expected);
    assert!(output.stderr.is_empty());
}
pub fn assert_error(output: &Output, expected: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains(expected),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
