use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

const CONFIG_EXAMPLE: &str = include_str!("../config.example.toml");

struct TemporaryHome(PathBuf);

impl TemporaryHome {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "daily-routine-cli-{}-{}",
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryHome {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn run(home: &TemporaryHome, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_daily-routine"))
        .args(args)
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", home.path())
        .output()
        .unwrap()
}

#[test]
fn self_check_succeeds_without_config_or_external_commands() {
    let home = TemporaryHome::new();

    let output = run(&home, &["--self-check", "--no-things"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "daily-routine self-check: ok\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn missing_config_exits_one_without_printing_a_report() {
    let home = TemporaryHome::new();

    let output = run(&home, &["--no-things"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.starts_with("failed to load configuration:"));
    assert!(stderr.ends_with(CONFIG_EXAMPLE));
}

#[test]
fn unknown_argument_exits_two_before_loading_config() {
    let home = TemporaryHome::new();

    let output = run(&home, &["--unknown"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("unknown argument: --unknown")
    );
}

#[test]
fn duplicate_argument_exits_two_before_loading_config() {
    let home = TemporaryHome::new();

    let output = run(&home, &["--no-things", "--no-things"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("duplicate argument: --no-things")
    );
}
