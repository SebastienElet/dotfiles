use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
fn repository() -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?)
}
fn invoke(home: &Path, args: &[&str]) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["eval", "--repository"])
        .arg(repository()?)
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", format!("{}:/usr/bin:/bin", home.display()))
        .output()?)
}
fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
#[test]
fn deterministic_operations_do_not_invoke_agents_and_smoke_observes_all_cases()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let home = tempfile::tempdir()?;
    for executable in ["codex", "claude", "curl", "bun"] {
        let path = home.path().join(executable);
        fs::write(&path, "#!/bin/sh\ntouch \"$HOME/agent-called\"\nexit 98\n")?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }
    success(&invoke(home.path(), &["validate-evals"])?);
    success(&invoke(home.path(), &["validate-evidence"])?);
    let smoke = invoke(home.path(), &["fixture-smoke"])?;
    success(&smoke);
    let report: Value = serde_json::from_slice(&smoke.stdout)?;
    assert_eq!(
        *(report).get("agent").ok_or("missing fixture index agent")?,
        "fixture-smoke"
    );
    let cases = (*(report).get("cases").ok_or("missing fixture index cases")?)
        .as_array()
        .ok_or("expected JSON array")?;
    assert_eq!(cases.len(), 3);
    for case in cases {
        assert_eq!(
            *(*(*(case).get("runs").ok_or("missing fixture index runs")?)
                .get(0)
                .ok_or("missing fixture index 0")?)
            .get("status")
            .ok_or("missing fixture index status")?,
            "PASS"
        );
    }
    assert_eq!(
        *(*(*(*(*(*(cases).first().ok_or("missing fixture index 0")?)
            .get("runs")
            .ok_or("missing fixture index runs")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("observations")
        .ok_or("missing fixture index observations")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("tool")
        .ok_or("missing fixture index tool")?,
        "cat"
    );
    assert_eq!(
        *(*(*(*(*(*(cases).first().ok_or("missing fixture index 0")?)
            .get("runs")
            .ok_or("missing fixture index runs")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("observations")
        .ok_or("missing fixture index observations")?)
        .get(1)
        .ok_or("missing fixture index 1")?)
        .get("tool")
        .ok_or("missing fixture index tool")?,
        "colgrep-search"
    );
    let path = home.path().join("smoke.json");
    fs::write(&path, &smoke.stdout)?;
    success(&invoke(
        home.path(),
        &[
            "validate-evidence",
            path.to_str().ok_or("required test value is missing")?,
        ],
    )?);
    let comparison = invoke(
        home.path(),
        &[
            "compare",
            path.to_str().ok_or("required test value is missing")?,
            path.to_str().ok_or("required test value is missing")?,
        ],
    )?;
    success(&comparison);
    let comparison: Value = serde_json::from_slice(&comparison.stdout)?;
    assert_eq!(
        *(comparison)
            .get("claim")
            .ok_or("missing fixture index claim")?,
        "descriptive-only"
    );
    assert_eq!(
        *(comparison)
            .get("passRateDelta")
            .ok_or("missing fixture index passRateDelta")?,
        0.0
    );
    assert!(!home.path().join("agent-called").exists());
    Ok(())
}
#[test]
fn run_refuses_existing_report_and_invalid_options_before_starting_agent()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let home = tempfile::tempdir()?;
    let report = home.path().join("report.json");
    fs::write(&report, "historical bytes")?;
    assert!(!invoke(home.path(), &["run"])?.status.success());
    for extra in [
        vec![],
        vec!["--runs", "0"],
        vec!["--only", "unknown"],
        vec!["--variant-file", "../../private.md"],
    ] {
        let mut args = vec![
            "run",
            "--model",
            "explicit",
            "--only",
            "code-search-literal",
            "--report",
            report.to_str().ok_or("required test value is missing")?,
        ];
        args.extend(extra);
        assert!(!invoke(home.path(), &args)?.status.success());
        assert_eq!(fs::read_to_string(&report)?, "historical bytes");
    }
    let missing = home.path().join("missing/report.json");
    let output = invoke(
        home.path(),
        &[
            "run",
            "--model",
            "explicit",
            "--only",
            "code-search-literal",
            "--report",
            missing.to_str().ok_or("required test value is missing")?,
        ],
    )?;
    assert!(!output.status.success());
    assert!(!missing.exists());
    Ok(())
}
fn fake_codex(
    home: &Path,
    terminal_event: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    fs::create_dir(home.join(".codex"))?;
    fs::write(home.join(".codex/auth.json"), "synthetic auth")?;
    let command = home.join("codex");
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'synthetic-codex-1\\n'; exit 0; fi\n/bin/cat >/dev/null\nrg FEATURE_FLAG_DISABLED >/dev/null\nprintf '%s\\n' '{terminal_event}'\n"
    );
    fs::write(&command, script)?;
    fs::set_permissions(command, fs::Permissions::from_mode(0o755))?;
    Ok(())
}
#[test]
fn manual_run_publishes_replicates_and_preserves_existing_history()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let home = tempfile::tempdir()?;
    fake_codex(
        home.path(),
        r#"{"type":"turn.completed","usage":{"input_tokens":12,"cached_input_tokens":2,"output_tokens":3}}"#,
    )?;
    let path = home.path().join("new.json");
    let args = [
        "run",
        "--model",
        "explicit",
        "--only",
        "code-search-literal",
        "--runs",
        "2",
        "--report",
        path.to_str().ok_or("required test value is missing")?,
    ];
    success(&invoke(home.path(), &args)?);
    let before = fs::read(&path)?;
    let report: Value = serde_json::from_slice(&before)?;
    assert_eq!(
        *(report).get("agent").ok_or("missing fixture index agent")?,
        "codex"
    );
    assert_eq!(
        *(report).get("model").ok_or("missing fixture index model")?,
        "explicit"
    );
    assert_eq!(
        *(report)
            .get("runCount")
            .ok_or("missing fixture index runCount")?,
        2
    );
    for run in (*(*(*(report).get("cases").ok_or("missing fixture index cases")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
    .get("runs")
    .ok_or("missing fixture index runs")?)
    .as_array()
    .ok_or("expected JSON array")?
    {
        assert_eq!(
            *(run).get("status").ok_or("missing fixture index status")?,
            "PASS"
        );
        assert_eq!(
            *(*(run).get("tokens").ok_or("missing fixture index tokens")?)
                .get("input")
                .ok_or("missing fixture index input")?,
            12
        );
        assert_eq!(
            *(*(*(run)
                .get("observations")
                .ok_or("missing fixture index observations")?)
            .get(0)
            .ok_or("missing fixture index 0")?)
            .get("tool")
            .ok_or("missing fixture index tool")?,
            "rg"
        );
    }
    assert!(!String::from_utf8_lossy(&before).contains("synthetic auth"));
    assert!(!invoke(home.path(), &args)?.status.success());
    assert_eq!(fs::read(path)?, before);
    Ok(())
}
#[test]
fn invalid_agent_output_publishes_invalid_run_and_returns_failure()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let home = tempfile::tempdir()?;
    fake_codex(home.path(), "not JSON")?;
    let path = home.path().join("invalid.json");
    let output = invoke(
        home.path(),
        &[
            "run",
            "--model",
            "explicit",
            "--only",
            "code-search-literal",
            "--report",
            path.to_str().ok_or("required test value is missing")?,
        ],
    )?;
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&fs::read(path)?)?;
    assert_eq!(
        *(*(*(*(*(report).get("cases").ok_or("missing fixture index cases")?)
            .get(0)
            .ok_or("missing fixture index 0")?)
        .get("runs")
        .ok_or("missing fixture index runs")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("status")
        .ok_or("missing fixture index status")?,
        "INVALID"
    );
    assert_eq!(
        *(*(*(*(*(report).get("cases").ok_or("missing fixture index cases")?)
            .get(0)
            .ok_or("missing fixture index 0")?)
        .get("runs")
        .ok_or("missing fixture index runs")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("error")
        .ok_or("missing fixture index error")?,
        "protocol-invalid"
    );
    Ok(())
}
#[test]
fn malformed_observation_is_retained_as_invalid_instead_of_aborting_report()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let home = tempfile::tempdir()?;
    fake_codex(
        home.path(),
        r#"{"type":"turn.completed","usage":{"input_tokens":12,"cached_input_tokens":2,"output_tokens":3}}"#,
    )?;
    let command = home.path().join("codex");
    let script = fs :: read_to_string (& command) ? . replace ("rg FEATURE_FLAG_DISABLED >/dev/null" , "printf '%s\\n' '{\"tool\":\"rg\",\"args\":[],\"exitCode\":9007199254740992}' > \"$HARNESS_EVAL_OBSERVATIONS\"" ,) ;
    fs::write(command, script)?;
    let path = home.path().join("invalid-observation.json");
    let output = invoke(
        home.path(),
        &[
            "run",
            "--model",
            "explicit",
            "--only",
            "code-search-literal",
            "--report",
            path.to_str().ok_or("required test value is missing")?,
        ],
    )?;
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&fs::read(path)?)?;
    assert_eq!(
        *(*(*(*(*(report).get("cases").ok_or("missing fixture index cases")?)
            .get(0)
            .ok_or("missing fixture index 0")?)
        .get("runs")
        .ok_or("missing fixture index runs")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("error")
        .ok_or("missing fixture index error")?,
        "observation-invalid"
    );
    assert_eq!(
        *(*(*(*(*(report).get("cases").ok_or("missing fixture index cases")?)
            .get(0)
            .ok_or("missing fixture index 0")?)
        .get("runs")
        .ok_or("missing fixture index runs")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("observations")
        .ok_or("missing fixture index observations")?,
        serde_json::json!([])
    );
    Ok(())
}
