mod support;

use std::fs;
use std::io::Read;
use std::os::unix::fs::{FileTypeExt, PermissionsExt};
use std::process::{Child, Command, Output, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use support::Fixture;

fn manifest() -> &'static str {
    "version: 1\nagents:\n  - id: codex\n    scopes: [user]\nstatuslines:\n  - { agent: codex, scope: user, items: [model] }\nresources: []\n"
}

fn spawn_statusline_doctor(fixture: &Fixture) -> Child {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["doctor", "statusline"])
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.home())
        .env("PATH", fixture.home().join("bin"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap()
}

fn wait_for_output(mut child: Child, timeout: Duration) -> Output {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            child
                .stdout
                .take()
                .unwrap()
                .read_to_end(&mut stdout)
                .unwrap();
            child
                .stderr
                .take()
                .unwrap()
                .read_to_end(&mut stderr)
                .unwrap();
            return Output {
                status,
                stdout,
                stderr,
            };
        }
        if started.elapsed() >= timeout {
            child.kill().unwrap();
            child.wait().unwrap();
            panic!("statusline doctor did not return before the deadline");
        }
        thread::sleep(Duration::from_millis(10));
    }
}

fn rendered(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8(output.stdout.clone()).unwrap(),
        String::from_utf8(output.stderr.clone()).unwrap()
    )
}

#[test]
fn fifo_configuration_is_rejected_without_opening_or_blocking() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", manifest());
    let configuration = fixture.home().join(".codex/config.toml");
    fs::create_dir_all(configuration.parent().unwrap()).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(&configuration)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        fs::symlink_metadata(&configuration)
            .unwrap()
            .file_type()
            .is_fifo()
    );

    let output = wait_for_output(spawn_statusline_doctor(&fixture), Duration::from_secs(2));

    assert_eq!(output.status.code(), Some(2));
    assert!(rendered(&output).contains("must be a regular file"));
}

#[test]
fn non_utf8_configuration_is_error_without_content_leak() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", manifest());
    let configuration = fixture.home().join(".codex/config.toml");
    fs::create_dir_all(configuration.parent().unwrap()).unwrap();
    fs::write(
        configuration,
        b"sensitive-non-utf8-marker = \"private\"\n[tui]\nstatus_line = [\"\xff\"]\n",
    )
    .unwrap();
    let before = fixture.snapshot();

    let output = fixture.command(["doctor", "statusline"]);
    let rendered = rendered(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(rendered.contains("Codex configuration is not UTF-8"));
    assert!(!rendered.contains("sensitive-non-utf8-marker"));
    assert!(!rendered.contains("private"));
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn unreadable_configuration_is_error_without_content_leak() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", manifest());
    let configuration = fixture.home().join(".codex/config.toml");
    fs::create_dir_all(configuration.parent().unwrap()).unwrap();
    fs::write(
        &configuration,
        "sensitive-unreadable-marker = \"private\"\n[tui]\nstatus_line = [\"model\"]\n",
    )
    .unwrap();
    fs::set_permissions(&configuration, fs::Permissions::from_mode(0o000)).unwrap();
    assert_eq!(
        fs::read(&configuration).unwrap_err().kind(),
        std::io::ErrorKind::PermissionDenied
    );

    let output = fixture.command(["doctor", "statusline"]);
    fs::set_permissions(&configuration, fs::Permissions::from_mode(0o600)).unwrap();
    let rendered = rendered(&output);

    assert_eq!(output.status.code(), Some(2));
    assert!(rendered.contains("could not read"));
    assert!(!rendered.contains("sensitive-unreadable-marker"));
    assert!(!rendered.contains("private"));
}

#[test]
fn missing_tui_table_is_drift_without_content_leak() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", manifest());
    fixture.write_home(
        ".codex/config.toml",
        "sensitive-missing-tui-marker = \"private\"\n[other]\nenabled = true\n",
    );

    let output = fixture.command(["doctor", "statusline"]);
    let rendered = rendered(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(rendered.contains("configuration is missing"));
    assert!(!rendered.contains("sensitive-missing-tui-marker"));
    assert!(!rendered.contains("private"));
}
