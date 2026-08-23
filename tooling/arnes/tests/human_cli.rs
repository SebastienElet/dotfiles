mod support;

use std::process::{Command, Output};
use support::Fixture;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(args)
        .env_clear()
        .output()
        .unwrap()
}

fn strip_ansi(value: &str) -> String {
    ["\x1b[31m", "\x1b[32m", "\x1b[33m", "\x1b[36m", "\x1b[0m"]
        .into_iter()
        .fold(value.to_owned(), |plain, sequence| {
            plain.replace(sequence, "")
        })
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
fn doctor_help_lists_color_options() {
    let output = run(&["doctor", "--help"]);
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("--color <COLOR>"), "{stdout}");
    for choice in ["auto", "always", "never"] {
        assert!(stdout.contains(choice), "{stdout}");
    }
    assert!(stdout.contains("[default: auto]"), "{stdout}");
    assert!(output.stderr.is_empty());
}

#[test]
fn verbose_json_is_rejected_before_home_is_read() {
    for args in [
        vec!["doctor", "skills", "-v", "--format", "json"],
        vec!["doctor", "--verbose", "skills", "--format=json"],
        vec!["doctor", "--format", "json", "skills", "-v"],
        vec![
            "doctor", "skills", "--color", "always", "-v", "--format", "json",
        ],
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
fn json_rejects_color_always_before_home_is_read() {
    for args in [
        vec!["doctor", "skills", "--color", "always", "--format", "json"],
        vec!["doctor", "--format=json", "--color=always", "skills"],
        vec!["doctor", "--color", "always", "--format", "json", "skills"],
    ] {
        let output = run(&args);

        assert_eq!(output.status.code(), Some(2), "{args:?}");
        assert!(output.stdout.is_empty(), "{args:?}");
        assert_eq!(
            String::from_utf8(output.stderr).unwrap(),
            "--color always cannot be used with --format json\n",
            "{args:?}"
        );
    }
}

#[test]
fn json_accepts_auto_and_never_without_ansi() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", "version: 1\nagents: []\nresources: []\n");

    for color in ["auto", "never"] {
        let output = fixture.command(["doctor", "manifest", "--format", "json", "--color", color]);

        assert_eq!(output.status.code(), Some(0), "{color}");
        assert!(!output.stdout.contains(&0x1b), "{color}");
        assert!(output.stderr.is_empty(), "{color}");
    }
}

#[test]
fn always_colors_redirected_output_and_overrides_no_color() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", "version: 1\nagents: []\nresources: []\n");
    let before = fixture.snapshot();
    let plain = fixture.command(["doctor", "manifest", "--color", "never"]);
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["doctor", "manifest", "--color", "always"])
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.home())
        .env("NO_COLOR", "1")
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.contains(&0x1b));
    assert_eq!(
        strip_ansi(&String::from_utf8(output.stdout).unwrap()),
        String::from_utf8(plain.stdout).unwrap()
    );
    assert!(output.stderr.is_empty());
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn redirected_auto_and_never_match_existing_plain_output() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", "version: 1\nagents: []\nresources: []\n");
    let before = fixture.snapshot();
    let default = fixture.command(["doctor", "manifest"]);

    for color in ["auto", "never"] {
        let output = fixture.command(["doctor", "manifest", "--color", color]);

        assert_eq!(output.status.code(), Some(0), "{color}");
        assert_eq!(output.stdout, default.stdout, "{color}");
        assert!(output.stderr.is_empty(), "{color}");
    }
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn duplicate_format_is_rejected_by_clap() {
    let output = run(&["doctor", "skills", "--format", "human", "--format", "json"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("cannot be used multiple times"), "{stderr}");
}

#[test]
fn duplicate_color_is_rejected_by_clap() {
    let output = run(&["doctor", "skills", "--color", "auto", "--color", "never"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("cannot be used multiple times"), "{stderr}");
}

#[test]
fn unknown_color_is_rejected_by_clap() {
    let output = run(&["doctor", "skills", "--color", "sometimes"]);
    let stderr = String::from_utf8(output.stderr).unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(stderr.contains("invalid value 'sometimes'"), "{stderr}");
}

#[test]
fn verbose_restores_healthy_details_before_or_after_the_resource() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", "version: 1\nagents: []\nresources: []\n");
    assert!(fixture.home().is_dir());
    assert!(fixture.repository().is_dir());
    let before = fixture.snapshot();

    let normal = fixture.command(["doctor", "manifest"]);
    let verbose_before = fixture.command(["doctor", "-v", "manifest"]);
    let verbose_after = fixture.command(["doctor", "manifest", "--verbose"]);

    assert_eq!(
        String::from_utf8(normal.stdout).unwrap(),
        "Manifest\n✓ 1 healthy\n"
    );
    for output in [verbose_before, verbose_after] {
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "Manifest\n✓ 1 healthy\n\nhealthy manifest: manifest is valid\n"
        );
        assert!(output.stderr.is_empty());
    }
    assert_eq!(fixture.snapshot(), before);
}
