use std::{fs, os::unix::fs::PermissionsExt, process::Command};

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn preflight_preserves_runtime_home_but_isolates_agent_home_without_exec() -> TestResult {
    for explicit in [false, true] {
        let home = tempfile::tempdir()?;
        let runtime = home
            .path()
            .join(if explicit { "custom-runtime" } else { ".volta" });
        fs::create_dir(&runtime)?;
        fs::write(runtime.join("installed"), "synthetic")?;
        fs::create_dir(home.path().join(".codex"))?;
        fs::write(home.path().join(".codex/auth.json"), "synthetic")?;
        let command = home.path().join("codex");
        fs::write(
            &command,
            r#"#!/bin/sh
[ -f "$VOLTA_HOME/installed" ] || exit 126
[ -f "$HOME/.codex/auth.json" ] || exit 127
[ "$CODEX_HOME" = "$HOME/.codex" ] || exit 128
[ ! -f "$HOME/codex" ] || exit 129
[ "$1" = --version ] || exit 130
printf 'synthetic-codex\n'
"#,
        )?;
        fs::set_permissions(&command, fs::Permissions::from_mode(0o755))?;
        let mut invocation = Command::new(env!("CARGO_BIN_EXE_arnes"));
        invocation
            .env_clear()
            .env("HOME", home.path())
            .env("PATH", home.path());
        if explicit {
            invocation.env("VOLTA_HOME", &runtime);
        }
        let output = invocation.args(["eval", "preflight"]).output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("synthetic-codex"));
    }
    Ok(())
}

#[test]
fn preflight_rejects_empty_version() -> TestResult {
    let home = tempfile::tempdir()?;
    fs::create_dir(home.path().join(".codex"))?;
    fs::write(home.path().join(".codex/auth.json"), "synthetic")?;
    let command = home.path().join("codex");
    fs::write(&command, "#!/bin/sh\nexit 0\n")?;
    fs::set_permissions(&command, fs::Permissions::from_mode(0o755))?;
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", home.path())
        .args(["eval", "preflight"])
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("empty version"));
    Ok(())
}
