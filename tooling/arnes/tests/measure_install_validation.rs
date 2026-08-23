use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
    executable: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let executable = root.path().join("bin/arnes");
        fs::create_dir(&home).unwrap();
        fs::create_dir(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"binary").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            _root: root,
            home,
            executable,
        }
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
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["measure", "install-hooks", "--agent", agent, "--command"])
            .arg(&self.executable)
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

#[path = "measure_install_validation/claude.rs"]
mod claude;
#[path = "measure_install_validation/codex.rs"]
mod codex;
#[path = "measure_install_validation/compatibility.rs"]
mod compatibility;
#[path = "measure_install_validation/cursor.rs"]
mod cursor;
