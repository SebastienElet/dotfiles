use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(args)
        .env_clear()
        .output()
        .unwrap()
}

#[test]
fn help_lists_doctor_resources() {
    let output = run(&["doctor", "--help"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).unwrap();
    for resource in [
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
    let output = run(&[
        "doctor", "skills", "--agent", "codex", "--scope", "project", "--format", "human",
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn json_doctor_emits_an_empty_diagnostic_list() {
    let output = run(&["doctor", "--format", "json"]);

    assert_eq!(output.status.code(), Some(0));
    assert_eq!(String::from_utf8(output.stdout).unwrap(), "[]\n");
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_values_exit_two_with_actionable_messages() {
    for (option, value, expected) in [
        ("", "unknown", "possible values: config"),
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
