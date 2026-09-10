#![cfg(test)]
#[path = "support/rules.rs"]
pub mod rule_support;
pub mod support;
use rule_support::{configured_fixture, link_rule, run};
use std::fs;
use std::os::unix::fs::symlink;
#[test]
fn missing_and_bad_sources_fail_closed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_file(
        fixture
            .repository()
            .join("harness/rules/agent-instructions.md"),
    )?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("source"));
    assert!(stdout.contains("is missing"));
    let fixture = configured_fixture()?;
    let source = fixture
        .repository()
        .join("harness/rules/agent-instructions.md");
    fs::remove_file(&source)?;
    fs::create_dir(&source)?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("is not a regular file"));
    Ok(())
}
#[test]
fn invalid_utf8_sources_fail_closed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::write(
        fixture
            .repository()
            .join("harness/rules/agent-instructions.md"),
        [0xff],
    )?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("could not be read as text"));
    Ok(())
}
#[test]
fn dangling_source_symlinks_fail_closed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let source = fixture
        .repository()
        .join("harness/rules/agent-instructions.md");
    fs::remove_file(&source)?;
    symlink("missing.md", &source)?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("missing (dangling symlink)"));
    Ok(())
}
#[test]
fn destination_failures_are_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination)?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 1);
    assert!(stdout.contains("destination"));
    assert!(stdout.contains("is missing"));
    let fixture = configured_fixture()?;
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination)?;
    fs::write(&destination, "copied\n")?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 1);
    assert!(stdout.contains("is not a symlink"));
    let fixture = configured_fixture()?;
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination)?;
    symlink("missing.md", &destination)?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 1);
    assert!(stdout.contains("is a dangling symlink"));
    Ok(())
}
#[test]
fn wrong_symlink_targets_are_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository("harness/rules/other.md", "other\n")?;
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::remove_file(&destination)?;
    link_rule(&fixture, "harness/rules/other.md")?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 1);
    assert!(stdout.contains("has the wrong symlink target"));
    Ok(())
}
#[test]
fn manifest_failures_are_rule_errors() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "rules"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("error rules: manifest: .arnes.yaml was not found"));
    assert!(stderr.is_empty());
    Ok(())
}
