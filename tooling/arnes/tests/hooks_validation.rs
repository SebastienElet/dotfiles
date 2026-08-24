use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let executable = home.join(".local/bin/arnes");
        fs::create_dir(&home).unwrap();
        fs::create_dir_all(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"binary").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        fs::write(
            home.join(".arnes.yaml"),
            "version: 1\nagents:\n  - id: claude\n    scopes: [user]\n  - id: cursor\n    scopes: [user]\n  - id: codex\n    scopes: [user]\nhooks:\n  - id: measurement\n    installations:\n      - { agent: claude, scope: user }\n      - { agent: cursor, scope: user }\n      - { agent: codex, scope: user }\nresources: []\n",
        )
        .unwrap();
        Self { _root: root, home }
    }

    fn config(&self, agent: &str) -> PathBuf {
        self.home.join(match agent {
            "codex" => ".codex/hooks.json",
            "claude-code" => ".claude/settings.json",
            "cursor" => ".cursor/hooks.json",
            _ => unreachable!(),
        })
    }

    fn install(&self, agent: &str) -> Output {
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

    fn write(&self, agent: &str, value: &Value) -> Vec<u8> {
        let path = self.config(agent);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = serde_json::to_vec(value).unwrap();
        fs::write(path, &bytes).unwrap();
        bytes
    }
}

fn nested(event: &str, handler: Value) -> Value {
    json!({"hooks": {event: [{"hooks": [handler]}]}})
}

fn direct(event: &str, handler: Value) -> Value {
    json!({"version": 1, "hooks": {event: [handler]}})
}

fn assert_failure_without_mutation(agent: &str, label: &str, config: Value) {
    let harness = Harness::new();
    let before = harness.write(agent, &config);
    let output = harness.install(agent);
    assert_eq!(output.status.code(), Some(2), "{label}");
    assert!(!output.stderr.is_empty(), "{label}");
    assert_eq!(fs::read(harness.config(agent)).unwrap(), before, "{label}");
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
