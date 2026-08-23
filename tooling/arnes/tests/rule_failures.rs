#[path = "support/rules.rs"]
pub mod rule_support;
pub mod support;

use rule_support::{configured_fixture, link_rule, run};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn missing_and_bad_sources_fail_closed() {
    let fixture = configured_fixture();
    fs::remove_file(
        fixture
            .repository()
            .join("harness/rules/agent-instructions.md"),
    )
    .unwrap();
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);
    assert_eq!(code, 2);
    assert!(stdout.contains("source"));
    assert!(stdout.contains("is missing"));

    let fixture = configured_fixture();
    let source = fixture
        .repository()
        .join("harness/rules/agent-instructions.md");
    fs::remove_file(&source).unwrap();
    fs::create_dir(&source).unwrap();
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);
    assert_eq!(code, 2);
    assert!(stdout.contains("is not a regular file"));
}

#[test]
fn invalid_utf8_sources_fail_closed() {
    let fixture = configured_fixture();
    fs::write(
        fixture
            .repository()
            .join("harness/rules/agent-instructions.md"),
        [0xff],
    )
    .unwrap();

    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);

    assert_eq!(code, 2);
    assert!(stdout.contains("could not be read as text"));
}

#[test]
fn dangling_source_symlinks_fail_closed() {
    let fixture = configured_fixture();
    let source = fixture
        .repository()
        .join("harness/rules/agent-instructions.md");
    fs::remove_file(&source).unwrap();
    symlink("missing.md", &source).unwrap();

    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);

    assert_eq!(code, 2);
    assert!(stdout.contains("missing (dangling symlink)"));
}

#[test]
fn destination_failures_are_drift() {
    let fixture = configured_fixture();
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination).unwrap();
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);
    assert_eq!(code, 1);
    assert!(stdout.contains("destination"));
    assert!(stdout.contains("is missing"));

    let fixture = configured_fixture();
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination).unwrap();
    fs::write(&destination, "copied\n").unwrap();
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);
    assert_eq!(code, 1);
    assert!(stdout.contains("is not a symlink"));

    let fixture = configured_fixture();
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination).unwrap();
    symlink("missing.md", &destination).unwrap();
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);
    assert_eq!(code, 1);
    assert!(stdout.contains("is a dangling symlink"));
}

#[test]
fn wrong_symlink_targets_are_drift() {
    let fixture = configured_fixture();
    fixture.write_repository("harness/rules/other.md", "other\n");
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination).unwrap();
    link_rule(&fixture, "harness/rules/other.md");

    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);

    assert_eq!(code, 1);
    assert!(stdout.contains("has the wrong symlink target"));
}

#[test]
fn manifest_failures_are_rule_errors() {
    let fixture = support::Fixture::new();

    let (code, stdout, stderr) = run(&fixture, &["doctor", "rules"]);

    assert_eq!(code, 2);
    assert!(stdout.contains("error rules: manifest: .arnes.yaml was not found"));
    assert!(stderr.is_empty());
}
