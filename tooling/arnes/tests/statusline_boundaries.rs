#![cfg(test)]
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
fn spawn_statusline_doctor(
    fixture: &Fixture,
) -> Result<Child, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["doctor", "statusline"])
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.home())
        .env("PATH", fixture.home().join("bin"))
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?)
}
fn wait_for_output(
    mut child: Child,
    timeout: Duration,
) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
    let started = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let mut stdout = Vec::new();
            let mut stderr = Vec::new();
            child
                .stdout
                .take()
                .ok_or("required test value is missing")?
                .read_to_end(&mut stdout)?;
            child
                .stderr
                .take()
                .ok_or("required test value is missing")?
                .read_to_end(&mut stderr)?;
            return Ok(Output {
                status,
                stdout,
                stderr,
            });
        }
        if started.elapsed() >= timeout {
            child.kill()?;
            child.wait()?;
            return Err("statusline doctor did not return before the deadline"
                .to_string()
                .into());
        }
        thread::sleep(Duration::from_millis(10));
    }
}
fn rendered(output: &Output) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(format!(
        "{}{}",
        String::from_utf8(output.stdout.clone())?,
        String::from_utf8(output.stderr.clone())?
    ))
}
#[test]
fn fifo_configuration_is_rejected_without_opening_or_blocking()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", manifest())?;
    let configuration = fixture.home().join(".codex/config.toml");
    fs::create_dir_all(configuration.parent().ok_or("fixture path has no parent")?)?;
    assert!(
        Command::new("mkfifo")
            .arg(&configuration)
            .status()?
            .success()
    );
    assert!(fs::symlink_metadata(&configuration)?.file_type().is_fifo());
    let output = wait_for_output(spawn_statusline_doctor(&fixture)?, Duration::from_secs(2))?;
    assert_eq!(output.status.code(), Some(2));
    assert!(rendered(&output)?.contains("must be a regular file"));
    Ok(())
}
#[test]
fn non_utf8_configuration_is_error_without_content_leak()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", manifest())?;
    let configuration = fixture.home().join(".codex/config.toml");
    fs::create_dir_all(configuration.parent().ok_or("fixture path has no parent")?)?;
    fs::write(
        configuration,
        b"sensitive-non-utf8-marker = \"private\"\n[tui]\nstatus_line = [\"\xff\"]\n",
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["doctor", "statusline"])?;
    let rendered = rendered(&output)?;
    assert_eq!(output.status.code(), Some(2));
    assert!(rendered.contains("Codex configuration is not UTF-8"));
    assert!(!rendered.contains("sensitive-non-utf8-marker"));
    assert!(!rendered.contains("private"));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
#[test]
fn unreadable_configuration_is_error_without_content_leak()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", manifest())?;
    let configuration = fixture.home().join(".codex/config.toml");
    fs::create_dir_all(configuration.parent().ok_or("fixture path has no parent")?)?;
    fs::write(
        &configuration,
        "sensitive-unreadable-marker = \"sensitive-unreadable-value\"\n[tui]\nstatus_line = [\"model\"]\n",
    )?;
    fs::set_permissions(&configuration, fs::Permissions::from_mode(0o000))?;
    assert_eq!(
        fs::read(&configuration)
            .err()
            .ok_or("unreadable fixture unexpectedly opened")?
            .kind(),
        std::io::ErrorKind::PermissionDenied
    );
    let output = fixture.command(["doctor", "statusline"])?;
    fs::set_permissions(&configuration, fs::Permissions::from_mode(0o600))?;
    let rendered = rendered(&output)?;
    assert_eq!(output.status.code(), Some(2));
    assert!(rendered.contains("could not read"));
    assert!(!rendered.contains("sensitive-unreadable-marker"));
    assert!(
        !rendered.contains("sensitive-unreadable-value"),
        "{rendered}"
    );
    Ok(())
}
#[test]
fn missing_tui_table_is_drift_without_content_leak()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", manifest())?;
    fixture.write_home(
        ".codex/config.toml",
        "sensitive-missing-tui-marker = \"private\"\n[other]\nenabled = true\n",
    )?;
    let output = fixture.command(["doctor", "statusline"])?;
    let rendered = rendered(&output)?;
    assert_eq!(output.status.code(), Some(1));
    assert!(rendered.contains("configuration is missing"));
    assert!(!rendered.contains("sensitive-missing-tui-marker"));
    assert!(!rendered.contains("private"));
    Ok(())
}
