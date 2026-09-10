#![cfg(test)]
use serde_json::Value;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use tempfile::TempDir;
struct Harness {
    _root: TempDir,
    home: PathBuf,
    executable: PathBuf,
}
impl Harness {
    fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_command_name("arnes")
    }
    fn with_command_name(name: &str) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let home = root.path().join(name);
        fs::create_dir(&home)?;
        let home = fs::canonicalize(home)?;
        let executable = home.join(".local/bin/arnes");
        fs::create_dir_all(executable.parent().ok_or("fixture path has no parent")?)?;
        fs::write(&executable, b"binary")?;
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700))?;
        let harness = Self {
            _root: root,
            home,
            executable,
        };
        harness.write_manifest(false)?;
        Ok(harness)
    }
    fn install(&self, agent: &str) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        self.setup(agent)
    }
    fn install_with(
        &self,
        agent: &str,
        command_path: &Path,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        fs::remove_file(&self.executable)?;
        symlink(command_path, &self.executable)?;
        self.setup(agent)
    }
    fn setup(&self, agent: &str) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.setup_command(agent).output()?)
    }
    fn setup_in_repository(
        &self,
        agent: &str,
        repository: &Path,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.setup_command(agent).current_dir(repository).output()?)
    }
    fn setup_command(&self, agent: &str) -> Command {
        let agent = if agent == "claude-code" {
            "claude"
        } else {
            agent
        };
        let mut command = Command::new(env!("CARGO_BIN_EXE_arnes"));
        command
            .args(["setup", "hooks", "--agent", agent])
            .env_clear()
            .env("HOME", &self.home);
        command
    }
    fn executable_named(
        &self,
        name: &str,
    ) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let path = self.home.join(".local/bin").join(name);
        fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
        fs::write(&path, b"binary")?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        Ok(path)
    }
    fn install_claude_with_handoff(
        &self,
        current: &Path,
        legacy: &Path,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        assert_eq!(
            legacy,
            current
                .parent()
                .ok_or("fixture path has no parent")?
                .parent()
                .ok_or("fixture path has no parent")?
                .join("scripts/agent_handoff")
        );
        let deployed = self.home.join(".local/bin/agent-handoff");
        symlink(current, deployed)?;
        self.write_manifest(true)?;
        let repository = current
            .parent()
            .ok_or("fixture path has no parent")?
            .parent()
            .ok_or("fixture path has no parent")?;
        self.setup_in_repository("claude-code", repository)
    }
    fn config(&self, agent: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let relative = match agent {
            "codex" => ".codex/hooks.json",
            "claude-code" => ".claude/settings.json",
            "cursor" => ".cursor/hooks.json",
            _ => return Err(format!("unsupported test agent: {agent}").into()),
        };
        Ok(self.home.join(relative))
    }
    fn command(&self, agent: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let path = self
            .executable
            .to_str()
            .ok_or("required test value is missing")?
            .replace('\'', "'\\''");
        Ok(format!("'{path}' measure hook --agent {agent}"))
    }
    fn memory_command(
        &self,
        agent: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let agent = if agent == "claude-code" {
            "claude"
        } else {
            agent
        };
        let path = self
            .home
            .join(".local/bin/agent-memory")
            .to_str()
            .ok_or("required test value is missing")?
            .replace('\'', "'\\''");
        Ok(format!("'{path}' hook --agent {agent}"))
    }
    fn write_memory_manifest(
        &self,
        agent: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let agent = if agent == "claude-code" {
            "claude"
        } else {
            agent
        };
        fs::write(
            self.home.join(".arnes.yaml"),
            format!(
                "version: 1\nagents:\n  - id: {agent}\n    scopes: [user]\nhooks:\n  - id: memory\n    installations:\n      - {{ agent: {agent}, scope: user }}\nresources: []\n"
            ),
        )?;
        Ok(())
    }
    fn write_manifest(
        &self,
        handoff: bool,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let handoff = if handoff {
            "  - id: handoff\n    installations:\n      - { agent: claude, scope: user }\n"
        } else {
            ""
        };
        fs::write(
            self.home.join(".arnes.yaml"),
            format!(
                "version: 1\nagents:\n  - id: claude\n    scopes: [user]\n  - id: cursor\n    scopes: [user]\n  - id: codex\n    scopes: [user]\nhooks:\n  - id: measurement\n    installations:\n      - {{ agent: claude, scope: user }}\n      - {{ agent: cursor, scope: user }}\n      - {{ agent: codex, scope: user }}\n{handoff}resources: []\n"
            ),
        )?;
        Ok(())
    }
    fn write_config(
        &self,
        agent: &str,
        value: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = self.config(agent)?;
        fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
        fs::write(path, serde_json::to_vec(value)?)?;
        Ok(())
    }
}
fn read_json(path: impl AsRef<Path>) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}
fn assert_failure(output: &Output) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
#[path = "hooks_reconciliation/filesystem.rs"]
mod filesystem;
#[path = "hooks_reconciliation/handoff.rs"]
mod handoff;
#[path = "hooks_reconciliation/installation.rs"]
mod installation;
#[path = "hooks_reconciliation/ownership.rs"]
mod ownership;
#[path = "hooks_reconciliation/validation.rs"]
mod validation;
