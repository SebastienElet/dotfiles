#![cfg(test)]
#[path = "support/rules.rs"]
pub mod rule_support;
pub mod support;
use rule_support::{configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;
#[test]
fn source_paths_cannot_escape_the_repository()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let source = fixture
        .repository()
        .join("harness/rules/agent-instructions.md");
    let outside = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside.md");
    fs::remove_file(&source)?;
    fs::write(&outside, "outside\n")?;
    symlink("../../../outside.md", &source)?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside the repository"));
    Ok(())
}
#[test]
fn destination_paths_cannot_escape_the_home_root()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let claude = fixture.home().join(".claude");
    let outside = fixture
        .home()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-claude");
    fs::rename(&claude, &outside)?;
    symlink("../outside-claude", &claude)?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside its declared root"));
    Ok(())
}
#[test]
fn missing_nested_destinations_remain_drift_within_home()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_dir_all(fixture.home().join(".claude"))?;
    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"])?;
    assert_eq!(code, 1);
    assert!(stdout.contains("is missing"));
    Ok(())
}
