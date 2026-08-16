#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use skill_support::{MANIFEST, configured_fixture, run};
use support::Fixture;

#[test]
fn supported_user_and_project_projections_are_managed() {
    let fixture = configured_fixture();
    let (code, stdout, stderr) = run(&fixture, &["doctor", "skills"]);

    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.lines().count(), 9);
    assert_eq!(stdout.matches("healthy skills: managed").count(), 9);
    for expected in [
        "managed claude user skill alpha",
        "managed claude project skill beta",
        "managed cursor user skill alpha",
        "managed cursor project skill beta",
        "managed codex user skill alpha",
        "managed codex project skill beta",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }
    assert!(stderr.is_empty());
}

#[test]
fn agent_and_scope_filters_isolate_skill_projections() {
    let fixture = configured_fixture();
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "project",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.lines().count(), 2);
    assert!(stdout.lines().all(|line| line.contains("cursor project")));

    let (code, stdout, _) = run(&fixture, &["doctor", "skills", "--scope", "user"]);
    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.lines().count(), 3);
    assert!(stdout.lines().all(|line| line.contains(" user ")));
}

#[test]
fn undeclared_and_unsupported_projections_are_reported() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    );
    let (code, stdout, _) = run(&fixture, &["doctor", "skills"]);
    assert_eq!(code, 0);
    assert!(stdout.contains("unsupported skills: claude user skill projection"));

    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        &MANIFEST.replace(".claude/skills/alpha", ".claude/other/alpha"),
    );
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("projection claude-user-alpha"));
    assert!(stdout.contains("is unsupported"));
}

#[test]
fn skills_doctor_is_isolated_and_read_only() {
    let fixture = configured_fixture();
    fixture.write_home("private", "temporary HOME sentinel");
    fixture.write_repository("private", "temporary repository sentinel");
    let before = fixture.snapshot();

    let (code, _, _) = run(&fixture, &["doctor", "skills"]);

    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn doctor_without_resource_does_not_aggregate_skills_yet() {
    let fixture = configured_fixture();
    let (code, stdout, _) = run(&fixture, &["doctor"]);

    assert_eq!(code, 0);
    assert_eq!(stdout, "healthy manifest: manifest is valid\n");
}
