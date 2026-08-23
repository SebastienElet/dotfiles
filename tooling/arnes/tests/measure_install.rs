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
        let home = root.path().join("home");
        let executable = root.path().join("bin").join(name);
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

    fn install(&self, agent: &str) -> Output {
        self.install_with(agent, &self.executable)
    }

    fn install_with(&self, agent: &str, command_path: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["measure", "install-hooks", "--agent", agent, "--command"])
            .arg(command_path)
            .env_clear()
            .env("HOME", &self.home)
            .output()
            .unwrap()
    }

    fn install_claude_with_handoff(&self, current: &Path, legacy: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args([
                "measure",
                "install-hooks",
                "--agent",
                "claude-code",
                "--command",
            ])
            .arg(&self.executable)
            .arg("--claude-stop-command")
            .arg(current)
            .arg("--claude-legacy-stop-command")
            .arg(legacy)
            .env_clear()
            .env("HOME", &self.home)
            .output()
            .unwrap()
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

#[path = "measure_install/filesystem.rs"]
mod filesystem;
#[path = "measure_install/installation.rs"]
mod installation;
#[path = "measure_install/ownership.rs"]
mod ownership;
#[path = "measure_install/validation.rs"]
mod validation;
