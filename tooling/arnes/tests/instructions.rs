#[path = "support/instructions.rs"]
pub mod instruction_support;
pub mod support;

use instruction_support::{configured_fixture, run};
use std::fs;
use support::Fixture;

#[test]
fn supported_projections_are_healthy_and_native_projections_are_unsupported() {
    let fixture = configured_fixture();
    let (code, stdout, stderr) = run(&fixture, &["doctor", "instructions"]);

    assert_eq!(code, 0);
    assert_eq!(stdout.lines().count(), 8);
    assert_eq!(stdout.matches("healthy instructions:").count(), 5);
    assert_eq!(stdout.matches("unsupported instructions:").count(), 3);
    for expected in [
        "healthy instructions: claude user instructions claude-user-instructions",
        "healthy instructions: claude user instructions claude-user-soul",
        "healthy instructions: claude user instructions claude-user-preferences",
        "healthy instructions: claude project instructions claude-project-instructions",
        "unsupported instructions: cursor user instruction projection",
        "unsupported instructions: cursor project instruction projection",
        "healthy instructions: codex user instructions codex-user-instructions",
        "unsupported instructions: codex project instruction projection",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }
    assert!(stderr.is_empty());
}

#[test]
fn agent_and_scope_filters_isolate_projections() {
    let fixture = configured_fixture();
    fixture.write_home(".codex/AGENTS.md", "stale\n");

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
        ],
    );
    assert_eq!(code, 0);
    assert_eq!(stdout.lines().count(), 3);
    assert!(!stdout.contains("codex"));

    let (code, stdout, _) = run(&fixture, &["doctor", "instructions", "--agent", "cursor"]);
    assert_eq!(code, 0);
    assert_eq!(stdout.matches("unsupported instructions:").count(), 2);

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "codex",
            "--scope",
            "project",
        ],
    );
    assert_eq!(code, 0);
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("unsupported instructions: codex project"));
}

#[test]
fn undeclared_filtered_combinations_are_unsupported() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    );
    let (code, stdout, stderr) = run(&fixture, &["doctor", "instructions", "--agent", "codex"]);

    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        "unsupported instructions: codex instruction projection is not declared or supported\n"
    );
    assert!(stderr.is_empty());
}

#[test]
fn codex_generated_content_must_match_declared_composition() {
    let fixture = configured_fixture();
    fixture.write_home(".codex/AGENTS.md", "rules\nsoul\nold user\n");
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "codex",
            "--scope",
            "user",
        ],
    );

    assert_eq!(code, 1);
    assert!(stdout.contains("generated file ~/.codex/AGENTS.md is stale"));
}

#[test]
fn repository_agents_file_has_precedence_for_claude_project() {
    let fixture = configured_fixture();
    fixture.write_repository("CLAUDE.md", "See @harness/AGENTS.md instead.\n");
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "project",
        ],
    );

    assert_eq!(code, 1);
    assert!(stdout.contains("does not include source AGENTS.md"));
}

#[test]
fn instructions_doctor_is_read_only() {
    let fixture = configured_fixture();
    let before = fixture.snapshot();

    let (code, _, _) = run(&fixture, &["doctor", "instructions"]);

    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn doctor_without_resource_does_not_aggregate_instructions_yet() {
    let fixture = configured_fixture();
    fs::remove_file(fixture.home().join(".codex/AGENTS.md")).unwrap();
    let (code, stdout, _) = run(&fixture, &["doctor"]);

    assert_eq!(code, 0);
    assert_eq!(stdout, "healthy manifest: manifest is valid\n");
}
