use serde_json::Value;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
    executable: PathBuf,
}

impl Harness {
    fn new() -> Self {
        Self::with_command_name("arnes")
    }

    fn with_command_name(name: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join(name);
        let executable = home.join(".local/bin/arnes");
        fs::create_dir(&home).unwrap();
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"binary").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        let harness = Self {
            _root: root,
            home,
            executable,
        };
        harness.write_manifest(false);
        harness
    }

    fn install(&self, agent: &str) -> Output {
        self.setup(agent)
    }

    fn install_with(&self, agent: &str, command_path: &Path) -> Output {
        fs::remove_file(&self.executable).unwrap();
        symlink(command_path, &self.executable).unwrap();
        self.setup(agent)
    }

    fn setup(&self, agent: &str) -> Output {
        let agent = if agent == "claude-code" {
            "claude"
        } else {
            agent
        };
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["setup", "hooks", "--agent", agent])
            .env_clear()
            .env("HOME", &self.home)
            .output()
            .unwrap()
    }

    fn install_claude_with_handoff(&self, current: &Path, legacy: &Path) -> Output {
        assert_eq!(
            legacy,
            current
                .parent()
                .unwrap()
                .parent()
                .unwrap()
                .join("scripts/agent_handoff")
        );
        let deployed = self.home.join(".local/bin/agent-handoff");
        symlink(current, deployed).unwrap();
        self.write_manifest(true);
        self.setup("claude-code")
    }

    fn config(&self, agent: &str) -> PathBuf {
        let relative = match agent {
            "codex" => ".codex/hooks.json",
            "claude-code" => ".claude/settings.json",
            "cursor" => ".cursor/hooks.json",
            _ => unreachable!(),
        };
        self.home.join(relative)
    }

    fn command(&self, agent: &str) -> String {
        let path = self.executable.to_str().unwrap().replace('\'', "'\\''");
        format!("'{path}' measure hook --agent {agent}")
    }

    fn write_manifest(&self, handoff: bool) {
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
        )
        .unwrap();
    }

    fn write_config(&self, agent: &str, value: &Value) {
        let path = self.config(agent);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    }
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
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
