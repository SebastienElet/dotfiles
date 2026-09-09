use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn invoke(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["eval", "--repository"])
        .arg(repository())
        .args(args)
        .env_clear()
        .env("HOME", home)
        .env("PATH", format!("{}:/usr/bin:/bin", home.display()))
        .output()
        .unwrap()
}

fn success(output: &Output) {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn deterministic_operations_do_not_invoke_agents_and_smoke_observes_all_cases() {
    let home = tempfile::tempdir().unwrap();
    for executable in ["codex", "claude", "curl", "bun"] {
        let path = home.path().join(executable);
        fs::write(&path, "#!/bin/sh\ntouch \"$HOME/agent-called\"\nexit 98\n").unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
    }
    success(&invoke(home.path(), &["validate-evals"]));
    success(&invoke(home.path(), &["validate-evidence"]));
    let smoke = invoke(home.path(), &["fixture-smoke"]);
    success(&smoke);
    let report: Value = serde_json::from_slice(&smoke.stdout).unwrap();
    assert_eq!(report["agent"], "fixture-smoke");
    let cases = report["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 3);
    for case in cases {
        assert_eq!(case["runs"][0]["status"], "PASS");
    }
    assert_eq!(cases[0]["runs"][0]["observations"][0]["tool"], "cat");
    assert_eq!(
        cases[0]["runs"][0]["observations"][1]["tool"],
        "colgrep-search"
    );
    let path = home.path().join("smoke.json");
    fs::write(&path, &smoke.stdout).unwrap();
    success(&invoke(
        home.path(),
        &["validate-evidence", path.to_str().unwrap()],
    ));
    let comparison = invoke(
        home.path(),
        &["compare", path.to_str().unwrap(), path.to_str().unwrap()],
    );
    success(&comparison);
    let comparison: Value = serde_json::from_slice(&comparison.stdout).unwrap();
    assert_eq!(comparison["claim"], "descriptive-only");
    assert_eq!(comparison["passRateDelta"], 0.0);
    assert!(!home.path().join("agent-called").exists());
}

#[test]
fn run_refuses_existing_report_and_invalid_options_before_starting_agent() {
    let home = tempfile::tempdir().unwrap();
    let report = home.path().join("report.json");
    fs::write(&report, "historical bytes").unwrap();
    assert!(!invoke(home.path(), &["run"]).status.success());
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
            report.to_str().unwrap(),
        ];
        args.extend(extra);
        assert!(!invoke(home.path(), &args).status.success());
        assert_eq!(fs::read_to_string(&report).unwrap(), "historical bytes");
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
            missing.to_str().unwrap(),
        ],
    );
    assert!(!output.status.success());
    assert!(!missing.exists());
}

fn fake_codex(home: &Path, terminal_event: &str) {
    fs::create_dir(home.join(".codex")).unwrap();
    fs::write(home.join(".codex/auth.json"), "synthetic auth").unwrap();
    let command = home.join("codex");
    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = --version ]; then printf 'synthetic-codex-1\\n'; exit 0; fi\n/bin/cat >/dev/null\nrg FEATURE_FLAG_DISABLED >/dev/null\nprintf '%s\\n' '{terminal_event}'\n"
    );
    fs::write(&command, script).unwrap();
    fs::set_permissions(command, fs::Permissions::from_mode(0o755)).unwrap();
}

#[test]
fn manual_run_publishes_replicates_and_preserves_existing_history() {
    let home = tempfile::tempdir().unwrap();
    fake_codex(
        home.path(),
        r#"{"type":"turn.completed","usage":{"input_tokens":12,"cached_input_tokens":2,"output_tokens":3}}"#,
    );
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
        path.to_str().unwrap(),
    ];
    success(&invoke(home.path(), &args));
    let before = fs::read(&path).unwrap();
    let report: Value = serde_json::from_slice(&before).unwrap();
    assert_eq!(report["agent"], "codex");
    assert_eq!(report["model"], "explicit");
    assert_eq!(report["runCount"], 2);
    for run in report["cases"][0]["runs"].as_array().unwrap() {
        assert_eq!(run["status"], "PASS");
        assert_eq!(run["tokens"]["input"], 12);
        assert_eq!(run["observations"][0]["tool"], "rg");
    }
    assert!(!String::from_utf8_lossy(&before).contains("synthetic auth"));
    assert!(!invoke(home.path(), &args).status.success());
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn invalid_agent_output_publishes_invalid_run_and_returns_failure() {
    let home = tempfile::tempdir().unwrap();
    fake_codex(home.path(), "not JSON");
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
            path.to_str().unwrap(),
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(report["cases"][0]["runs"][0]["status"], "INVALID");
    assert_eq!(report["cases"][0]["runs"][0]["error"], "protocol-invalid");
}

#[test]
fn malformed_observation_is_retained_as_invalid_instead_of_aborting_report() {
    let home = tempfile::tempdir().unwrap();
    fake_codex(
        home.path(),
        r#"{"type":"turn.completed","usage":{"input_tokens":12,"cached_input_tokens":2,"output_tokens":3}}"#,
    );
    let command = home.path().join("codex");
    let script = fs::read_to_string(&command).unwrap().replace(
        "rg FEATURE_FLAG_DISABLED >/dev/null",
        "printf '%s\\n' '{\"tool\":\"rg\",\"args\":[],\"exitCode\":9007199254740992}' > \"$HARNESS_EVAL_OBSERVATIONS\"",
    );
    fs::write(command, script).unwrap();
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
            path.to_str().unwrap(),
        ],
    );
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(
        report["cases"][0]["runs"][0]["error"],
        "observation-invalid"
    );
    assert_eq!(
        report["cases"][0]["runs"][0]["observations"],
        serde_json::json!([])
    );
}
