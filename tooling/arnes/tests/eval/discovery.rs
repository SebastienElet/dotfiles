use std::{
    fs,
    os::unix::fs::{PermissionsExt, symlink},
    path::Path,
    process::Command,
};

fn run(
    script: &str,
    linked: bool,
    relative_path: bool,
) -> Result<(tempfile::TempDir, std::process::Output), Box<dyn std::error::Error + Send + Sync>> {
    let home = tempfile::tempdir()?;
    fs::create_dir(home.path().join(".codex"))?;
    fs::write(home.path().join(".codex/auth.json"), "synthetic auth")?;
    fs::create_dir(home.path().join("bin"))?;
    let executable = home
        .path()
        .join(if linked { "bin/shim" } else { "bin/codex" });
    fs::write(&executable, script)?;
    fs::set_permissions(&executable, fs::Permissions::from_mode(0o755))?;
    if linked {
        symlink("shim", home.path().join("bin/codex"))?;
    }
    let path = if relative_path {
        "bin:/usr/bin:/bin".to_owned()
    } else {
        format!("{}/bin:/usr/bin:/bin", home.path().display())
    };
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .current_dir(home.path())
        .env_clear()
        .env("HOME", home.path())
        .env("PATH", path)
        .args(["eval", "--repository"])
        .arg(Path::new(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .args([
            "run",
            "--model",
            "synthetic",
            "--only",
            "code-search-literal",
            "--runs",
            "1",
            "--report",
            "report.json",
        ])
        .output()?;
    Ok((home, output))
}

#[test]
fn preserves_shim_invocation_name_for_absolute_and_relative_path_entries()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let script = r#"#!/bin/sh
[ "${0##*/}" = codex ] || exit 126
if [ "$1" = --version ]; then printf 'synthetic-codex\n'; exit 0; fi
/bin/cat >/dev/null
rg FEATURE_FLAG_DISABLED >/dev/null
printf '%s\n' '{"type":"turn.completed","usage":{"input_tokens":1,"cached_input_tokens":0,"output_tokens":1}}'
"#;
    for relative in [false, true] {
        let (home, output) = run(script, true, relative)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: serde_json::Value =
            serde_json::from_slice(&fs::read(home.path().join("report.json"))?)?;
        assert_eq!(
            report
                .pointer("/cases/0/runs/0/status")
                .and_then(serde_json::Value::as_str),
            Some("PASS")
        );
    }
    Ok(())
}

#[test]
fn discovery_reports_launch_exit_and_timeout_failures_without_publishing()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (script, expected) in [
        ("#!/nonexistent-interpreter\n", "could not start process"),
        (
            "#!/bin/sh\nprintf private-sentinel >&2\nexit 126\n",
            "process exited with code 126",
        ),
        ("#!/bin/sh\nexec /bin/sleep 10\n", "process timed out"),
    ] {
        let (home, output) = run(script, false, false)?;
        assert!(!output.status.success());
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(expected), "{error}");
        assert!(!error.contains("private-sentinel"));
        assert!(!home.path().join("report.json").exists());
    }
    Ok(())
}
