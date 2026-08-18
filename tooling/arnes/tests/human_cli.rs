use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(args)
        .env_clear()
        .output()
        .unwrap()
}

#[test]
fn doctor_help_lists_verbose_options() {
    let output = run(&["doctor", "--help"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("-v, --verbose"), "{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn verbose_json_is_rejected_before_home_is_read() {
    for args in [
        vec!["doctor", "skills", "-v", "--format", "json"],
        vec!["doctor", "--verbose", "skills", "--format=json"],
        vec!["doctor", "--format", "json", "skills", "-v"],
    ] {
        let output = run(&args);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "--verbose cannot be used with --format json\n",
            "{args:?}"
        );
    }
}

#[test]
fn duplicate_format_is_rejected_by_clap() {
    let output = run(&[
        "doctor", "skills", "--format", "human", "--format", "json",
    ]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("cannot be used multiple times"), "{stderr}");
}
