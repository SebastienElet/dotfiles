#![cfg(test)]
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;
struct Harness {
    _root: TempDir,
    home: PathBuf,
    repository: PathBuf,
}
impl Harness {
    fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let home = root.path().join("home");
        let repository = root.path().join("repository");
        let executable = home.join(".local/bin/arnes");
        fs::create_dir(&home)?;
        fs::create_dir(&repository)?;
        let repository = fs::canonicalize(repository)?;
        fs::create_dir_all(executable.parent().ok_or("fixture path has no parent")?)?;
        fs::write(&executable, b"binary")?;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))?;
        fs::write(
            home.join(".arnes.yaml"),
            "version: 1\nagents:\n  - id: claude\n    scopes: [user]\n  - id: cursor\n    scopes: [user]\n  - id: codex\n    scopes: [user]\nhooks:\n  - id: measurement\n    installations:\n      - { agent: claude, scope: user }\n      - { agent: cursor, scope: user }\n      - { agent: codex, scope: user }\nresources: []\n",
        )?;
        Ok(Self {
            _root: root,
            home,
            repository,
        })
    }
    fn config(&self, agent: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.home.join(match agent {
            "codex" => ".codex/hooks.json",
            "claude-code" => ".claude/settings.json",
            "cursor" => ".cursor/hooks.json",
            _ => return Err(format!("unsupported test agent: {agent}").into()),
        }))
    }
    fn install(&self, agent: &str) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        let agent = if agent == "claude-code" {
            "claude"
        } else {
            agent
        };
        Ok(Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["setup", "hooks", "--agent", agent])
            .env_clear()
            .env("HOME", &self.home)
            .current_dir(&self.repository)
            .output()?)
    }
    fn write(
        &self,
        agent: &str,
        value: &Value,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let path = self.config(agent)?;
        fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
        let bytes = serde_json::to_vec(value)?;
        fs::write(path, &bytes)?;
        Ok(bytes)
    }
}
fn nested(event: &str, handler: &Value) -> Value {
    json ! ({ "hooks" : { event : [{ "hooks" : [handler] }] } })
}
fn direct(event: &str, handler: &Value) -> Value {
    json ! ({ "version" : 1 , "hooks" : { event : [handler] } })
}
fn assert_failure_without_mutation(
    agent: &str,
    label: &str,
    config: &Value,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let before = harness.write(agent, config)?;
    let output = harness.install(agent)?;
    assert_eq!(output.status.code(), Some(2), "{label}");
    assert!(!output.stderr.is_empty(), "{label}");
    assert_eq!(fs::read(harness.config(agent)?)?, before, "{label}");
    Ok(())
}
fn assert_success(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[path = "hooks_validation/claude.rs"]
mod claude;
#[path = "hooks_validation/codex.rs"]
mod codex;
#[path = "hooks_validation/compatibility.rs"]
mod compatibility;
#[path = "hooks_validation/cursor.rs"]
mod cursor;
