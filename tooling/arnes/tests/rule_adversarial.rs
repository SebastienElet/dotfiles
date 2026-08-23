#[path = "support/rules.rs"]
pub mod rule_support;
pub mod support;

use rule_support::{configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn source_paths_cannot_escape_the_repository() {
    let fixture = configured_fixture();
    let source = fixture
        .repository()
        .join("harness/rules/agent-instructions.md");
    let outside = fixture.repository().parent().unwrap().join("outside.md");
    fs::remove_file(&source).unwrap();
    fs::write(&outside, "outside\n").unwrap();
    symlink("../../../outside.md", &source).unwrap();

    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);

    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside the repository"));
}

#[test]
fn destination_paths_cannot_escape_the_home_root() {
    let fixture = configured_fixture();
    let claude = fixture.home().join(".claude");
    let outside = fixture.home().parent().unwrap().join("outside-claude");
    fs::rename(&claude, &outside).unwrap();
    symlink("../outside-claude", &claude).unwrap();

    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);

    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside its declared root"));
}

#[test]
fn missing_nested_destinations_remain_drift_within_home() {
    let fixture = configured_fixture();
    fs::remove_dir_all(fixture.home().join(".claude")).unwrap();

    let (code, stdout, _) = run(&fixture, &["doctor", "rules", "--agent", "claude"]);

    assert_eq!(code, 1);
    assert!(stdout.contains("is missing"));
}
