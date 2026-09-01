#[path = "support/rules.rs"]
pub mod rule_support;
pub mod support;

use rule_support::{configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn claude_user_rule_symlinks_are_healthy() {
    let fixture = configured_fixture();

    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "rules", "--agent", "claude", "--scope", "user", "-v",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy rules: claude user rule agent-instructions"));
    assert!(stdout.contains("destination ~/.claude/rules/agent-instructions.md is current"));
    assert!(stderr.is_empty());
}

#[test]
fn cursor_user_rule_symlinks_are_healthy() {
    let fixture = support::Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: cursor\n    scopes: [user]\nresources:\n  - id: memory-governance-cursor\n    kind: rules\n    agent: cursor\n    scope: user\n    source: { root: repository, path: harness/rules/memory-governance-cursor.mdc }\n    destination: { root: home, path: .cursor/rules/memory-governance-cursor.mdc }\n",
    );
    fixture.write_repository(
        "harness/rules/memory-governance-cursor.mdc",
        "---\nalwaysApply: true\n---\n",
    );
    let destination = fixture
        .home()
        .join(".cursor/rules/memory-governance-cursor.mdc");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    symlink(
        fs::canonicalize(
            fixture
                .repository()
                .join("harness/rules/memory-governance-cursor.mdc"),
        )
        .unwrap(),
        destination,
    )
    .unwrap();

    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "rules", "--agent", "cursor", "--scope", "user", "-v",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy rules: cursor user rule memory-governance-cursor"));
    assert!(stdout.contains("destination ~/.cursor/rules/memory-governance-cursor.mdc is current"));
    assert!(stderr.is_empty());
}

#[test]
fn filters_isolate_rules_and_report_unsupported_capabilities() {
    let fixture = configured_fixture();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "codex", "--scope", "user"],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("codex user rule projection is not declared or supported"));
    assert!(!stdout.contains("agent-instructions"));

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "claude", "--scope", "project"],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("claude project rule projection is not declared or supported"));

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "codex", "--scope", "project"],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("codex project rule projection is not declared or supported"));
}

#[test]
fn undeclared_filters_are_explicitly_unsupported() {
    let fixture = support::Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    );

    let (code, stdout, stderr) = run(&fixture, &["doctor", "rules", "--agent", "codex"]);

    assert_eq!(code, 0);
    assert!(stdout.contains("unsupported rules: codex user rule projection"));
    assert!(stderr.is_empty());
}

#[test]
fn rule_diagnostics_use_the_rules_resource() {
    let fixture = configured_fixture();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "claude", "--format", "json"],
    );

    assert_eq!(code, 0);
    let diagnostics: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(diagnostics[0]["resource"], "rules");
    assert_eq!(diagnostics[0]["state"], "healthy");
}

#[test]
fn rules_doctor_is_read_only() {
    let fixture = configured_fixture();
    let before = fixture.snapshot();

    let (code, _, _) = run(&fixture, &["doctor", "rules"]);

    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot(), before);
}
