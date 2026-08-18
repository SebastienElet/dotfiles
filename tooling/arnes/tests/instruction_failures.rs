#[path = "support/instructions.rs"]
pub mod instruction_support;
pub mod support;

use instruction_support::{configured_fixture, remove, replace_home_link, run};
use std::fs;

#[test]
fn missing_sources_fail_closed_and_missing_destinations_drift() {
    let fixture = configured_fixture();
    remove(fixture.repository().join("harness/SOUL.md"));
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
            "-v",
        ],
    );
    assert_eq!(code, 2);
    assert!(stdout.contains("source"));
    assert!(stdout.contains("harness/SOUL.md"));
    assert!(stdout.contains("is missing"));

    let fixture = configured_fixture();
    remove(fixture.home().join(".claude/CLAUDE.md"));
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
    assert_eq!(code, 1);
    assert!(stdout.contains("destination"));
    assert!(stdout.contains("is missing"));

    let fixture = configured_fixture();
    remove(fixture.home().join(".claude/SOUL.md"));
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
    assert_eq!(code, 1);
    assert!(stdout.contains("include"));
    assert!(stdout.contains(".claude/SOUL.md"));
    assert!(stdout.contains("is missing"));
}

#[test]
fn wrong_symlink_targets_are_drift() {
    let fixture = configured_fixture();
    replace_home_link(&fixture, "harness/USER.md", ".claude/SOUL.md");
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

    assert_eq!(code, 1);
    assert!(stdout.contains("wrong symlink target"));
    assert!(stdout.contains(".claude/SOUL.md"));
}

#[test]
fn missing_includes_and_cycles_are_errors() {
    let fixture = configured_fixture();
    fixture.write_repository("harness/AGENTS.md", "@MISSING.md\n");
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
    assert_eq!(code, 2);
    assert!(stdout.contains("include"));
    assert!(stdout.contains("MISSING.md"));
    assert!(stdout.contains("is missing"));

    let fixture = configured_fixture();
    fixture.write_repository("harness/SOUL.md", "@CLAUDE.md\n");
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
    assert_eq!(code, 2);
    assert!(stdout.contains("include cycle"));
}

#[test]
fn includes_resolve_from_the_effective_destination() {
    let fixture = configured_fixture();
    let manifest = fs::read_to_string(fixture.home().join(".arnes.yaml")).unwrap();
    fixture.write_home(
        ".arnes.yaml",
        &manifest.replace(".claude/SOUL.md", ".claude/nested/SOUL.md"),
    );
    remove(fixture.home().join(".claude/SOUL.md"));
    instruction_support::link_home(&fixture, "harness/SOUL.md", ".claude/nested/SOUL.md");
    fixture.write_repository("harness/AGENTS.md", "@nested/SOUL.md\n@USER.md\nrules\n");
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
            "-v",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.matches("healthy instructions:").count(), 3);
}

#[test]
fn includes_cannot_escape_their_fixture_root() {
    let fixture = configured_fixture();
    fixture.write_repository("harness/AGENTS.md", "@../../outside.md\n");
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

    assert_eq!(code, 2);
    assert!(stdout.contains("escapes its instruction root"));
}

#[test]
fn malformed_instruction_paths_fail_closed() {
    let fixture = configured_fixture();
    fs::create_dir(fixture.repository().join("harness/BROKEN.md")).unwrap();
    fixture.write_repository("harness/AGENTS.md", "@BROKEN.md\n");
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

    assert_eq!(code, 2);
    assert!(stdout.contains("is not a file"));
}

#[test]
fn manifest_failures_fail_closed_as_instruction_errors() {
    let fixture = support::Fixture::new();
    let (code, stdout, stderr) = run(&fixture, &["doctor", "instructions"]);

    assert_eq!(code, 2);
    assert_eq!(
        stdout,
        "Instructions · user scope\n✓ 0 healthy\n\nerror instructions: manifest: .arnes.yaml was not found\n"
    );
    assert!(stderr.is_empty());
}
