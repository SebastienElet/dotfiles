mod support;

use std::path::Path;
use std::process::{Command, Output};
use support::Fixture;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(args)
        .env_clear()
        .output()
        .unwrap()
}

fn run_with_home(args: &[&str], home: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(args)
        .env_clear()
        .env("HOME", home)
        .output()
        .unwrap()
}

fn manifest(name: &str) -> String {
    std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/manifest")
            .join(name),
    )
    .unwrap()
}

#[test]
fn help_lists_doctor_resources() {
    let output = run(&["doctor", "--help"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for resource in [
        "manifest",
        "config",
        "instructions",
        "skills",
        "prompts",
        "commands",
        "rules",
        "hooks",
        "mcp",
        "statusline",
    ] {
        assert!(stdout.contains(resource), "help omits {resource}: {stdout}");
    }
}

#[test]
fn version_succeeds() {
    let output = run(&["--version"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "arnes 0.1.0\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn doctor_accepts_shared_options_without_reading_the_environment() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: codex\n    scopes: [project]\nresources: []\n",
    );
    let output = fixture.command([
        "doctor", "skills", "--agent", "codex", "--scope", "project", "--format", "human",
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "unsupported skills: codex project skill projection is not declared or supported\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn json_doctor_emits_the_manifest_diagnostic() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("valid.yaml"));
    let output = fixture.command(["doctor", "--format", "json"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "[\n  {\n    \"resource\": \"manifest\",\n    \"state\": \"healthy\",\n    \"message\": \"manifest is valid\"\n  }\n]\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn manifest_doctor_loads_from_the_injected_home() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("valid.yaml"));
    let output = fixture.command(["doctor", "manifest"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "healthy manifest: manifest is valid\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn manifest_doctor_reports_invalid_manifests() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("unsupported-version.yaml"));
    let output = fixture.command(["doctor", "manifest"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "error manifest: version: unsupported version 2; expected 1\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn operational_failures_use_json_and_exit_two() {
    let fixture = Fixture::new();
    let output = fixture.command(["doctor", "manifest", "--format", "json"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "[\n  {\n    \"resource\": \"manifest\",\n    \"state\": \"error\",\n    \"message\": \"manifest: .arnes.yaml was not found\"\n  }\n]\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn output_failures_exit_two_instead_of_passing_silently() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("valid.yaml"));
    let output = Command::new("sh")
        .args(["-c", "exec 1<\"$2\"; exec \"$1\" doctor manifest", "sh"])
        .arg(env!("CARGO_BIN_EXE_arnes"))
        .arg(fixture.home().join(".arnes.yaml"))
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.home())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("output: could not write diagnostics:")
    );
}

#[test]
fn manifest_doctor_requires_home_without_reading_the_environment() {
    let output = run(&["doctor", "manifest"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "error manifest: HOME: environment variable is required\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn skills_doctor_requires_injected_home_without_fallback() {
    let output = run(&["doctor", "skills"]);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "error skills: HOME: environment variable is required\n"
    );
    assert!(output.stderr.is_empty());
}

#[test]
fn manifest_doctor_rejects_home_paths_relative_to_the_repository() {
    for (home, message) in [
        ("", "HOME: environment variable cannot be empty"),
        (
            "fixture/home",
            "HOME: environment variable must be an absolute path",
        ),
    ] {
        let output = run_with_home(&["doctor", "manifest"], home);

        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            format!("error manifest: {message}\n")
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn fixture_run_is_isolated_and_read_only() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("valid.yaml"));
    fixture.write_home("private", "home sentinel");
    fixture.write_repository("private", "repository sentinel");
    let before = fixture.snapshot();

    let output = fixture.command(["doctor", "manifest"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn invalid_values_exit_two_with_actionable_messages() {
    for (option, value, expected) in [
        ("", "unknown", "possible values: manifest"),
        ("--agent", "unknown", "possible values: claude"),
        ("--scope", "unknown", "possible values: user"),
        ("--format", "unknown", "possible values: human"),
    ] {
        let mut args = vec!["doctor"];
        if !option.is_empty() {
            args.push(option);
        }
        args.push(value);

        let output = run(&args);

        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("invalid value 'unknown'"), "{stderr}");
        assert!(stderr.contains(expected), "{stderr}");
    }
}
