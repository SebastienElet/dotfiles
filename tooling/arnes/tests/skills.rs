#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use skill_support::{MANIFEST, configured_fixture, run};
use support::Fixture;

#[test]
fn user_scope_is_default_for_declared_skills() {
    let fixture = configured_fixture();
    let (code, stdout, stderr) = run(&fixture, &["doctor", "skills", "-v"]);

    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.matches("  HEALTHY alpha").count(), 3);
    assert!(!stdout.contains(" project "));
    for expected in ["CLAUDE", "CURSOR", "CODEX"] {
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
            "doctor", "skills", "--agent", "cursor", "--scope", "project", "-v",
        ],
    );

    assert_eq!(code, 1, "{stdout}");
    assert_eq!(
        stdout
            .lines()
            .filter(|line| line.trim_start().starts_with("HEALTHY project-alpha"))
            .count(),
        1
    );
    assert!(
        stdout
            .lines()
            .any(|line| line.contains("DRIFT broken managed cursor project skill beta")),
        "{stdout}"
    );
    assert!(!stdout.lines().any(|line| line.contains("HEALTHY alpha")));
    assert!(stdout.starts_with("Skills · project scope · cursor agent"));

    let (code, stdout, _) = run(&fixture, &["doctor", "skills", "--scope", "user"]);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.starts_with("Skills · user scope · 3 agents"));
    assert!(!stdout.contains(" project "));
    assert!(!stdout.contains("project-alpha"));
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
    assert!(stdout.contains("UNSUPPORTED claude user skill projection"));

    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        &MANIFEST.replacen(
            "source: { root: repository, path: harness/skills }",
            "source: { root: repository, path: .agents/other }",
            1,
        ),
    );
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );
    assert_eq!(code, 0);
    assert!(stdout.contains("projection claude-user-skills"));
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
fn doctor_without_resource_reports_manifest_then_skills() {
    let fixture = configured_fixture();
    let (code, stdout, _) = run(&fixture, &["doctor"]);

    assert_eq!(code, 0);
    assert!(
        stdout.starts_with("Manifest\n✓ 1 healthy\n\nSkills · user scope · 3 agents\n"),
        "{stdout}"
    );
}

#[test]
fn doctor_without_resource_fails_when_skills_drift() {
    let fixture = configured_fixture();
    std::fs::remove_file(fixture.home().join(".cursor/skills/alpha")).unwrap();

    let (code, stdout, _) = run(&fixture, &["doctor"]);

    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.starts_with("Manifest\n✓ 1 healthy\n\nSkills · user scope"));
    assert!(stdout.contains("DRIFT alpha"));
}

#[test]
fn json_doctor_without_resource_includes_manifest_and_skills() {
    let fixture = configured_fixture();
    let (code, stdout, _) = run(&fixture, &["doctor", "--format", "json"]);
    let diagnostics: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let diagnostics = diagnostics.as_array().unwrap();

    assert_eq!(code, 0, "{stdout}");
    assert_eq!(diagnostics[0]["resource"], "manifest");
    assert!(
        diagnostics
            .iter()
            .skip(1)
            .all(|diagnostic| diagnostic["resource"] == "skills")
    );
    assert!(diagnostics.len() > 1);
}

#[test]
fn doctor_without_resource_stops_after_an_invalid_manifest() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        &MANIFEST.replacen("version: 1", "version: 2", 1),
    );

    let (code, stdout, stderr) = run(&fixture, &["doctor"]);

    assert_eq!(code, 2);
    assert_eq!(
        stdout,
        "Manifest\n✓ 0 healthy\n\nerror manifest: version: unsupported version 2; expected 1\n"
    );
    assert!(stderr.is_empty());
}
