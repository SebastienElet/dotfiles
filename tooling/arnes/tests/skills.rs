#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;
use skill_support::{MANIFEST, configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;
use support::Fixture;
#[test]
fn user_scope_is_default_for_declared_skills()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "skills", "-v"])?;
    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.matches("  HEALTHY alpha").count(), 3);
    assert!(!stdout.contains(" project "));
    for expected in ["CLAUDE", "CURSOR", "CODEX"] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn user_skill_targets_from_the_deployed_checkout_are_healthy_from_a_worktree()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository("home/.arnes.yaml", MANIFEST)?;
    fs::remove_file(fixture.home().join(".arnes.yaml"))?;
    symlink(
        fixture.repository().join("home/.arnes.yaml"),
        fixture.home().join(".arnes.yaml"),
    )?;
    let worktree = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("worktree");
    create_worktree(fixture.repository(), &worktree)?;
    let before = fixture.snapshot()?;
    let output = fixture.command_from(
        &worktree,
        ["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("✓ 1 healthy"), "{stdout}");
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
fn create_worktree(
    repository: &Path,
    worktree: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for arguments in [
        vec!["init", "--quiet"],
        vec!["add", "."],
        vec![
            "-c",
            "user.name=Arnes Test",
            "-c",
            "user.email=arnes@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "fixture",
        ],
    ] {
        assert!(
            Command::new("git")
                .args(arguments)
                .current_dir(repository)
                .status()?
                .success()
        );
    }
    assert!(
        Command::new("git")
            .args(["worktree", "add", "--quiet", "--detach"])
            .arg(worktree)
            .current_dir(repository)
            .status()?
            .success()
    );
    Ok(())
}
#[test]
fn agent_and_scope_filters_isolate_skill_projections()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "project", "-v",
        ],
    )?;
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
    let (code, stdout, _) = run(&fixture, &["doctor", "skills", "--scope", "user"])?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.starts_with("Skills · user scope · 3 agents"));
    assert!(!stdout.contains(" project "));
    assert!(!stdout.contains("project-alpha"));
    Ok(())
}
#[test]
fn undeclared_and_unsupported_projections_are_reported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    )?;
    let (code, stdout, _) = run(&fixture, &["doctor", "skills"])?;
    assert_eq!(code, 0);
    assert!(stdout.contains("UNSUPPORTED claude user skill projection"));
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        &MANIFEST.replacen(
            "source: { root: repository, path: harness/skills }",
            "source: { root: repository, path: .agents/other }",
            1,
        ),
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 0);
    assert!(stdout.contains("projection claude-user-skills"));
    assert!(stdout.contains("is unsupported"));
    Ok(())
}
#[test]
fn skills_doctor_is_isolated_and_read_only() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    fixture.write_home("private", "temporary HOME sentinel")?;
    fixture.write_repository("private", "temporary repository sentinel")?;
    let before = fixture.snapshot()?;
    let (code, _, _) = run(&fixture, &["doctor", "skills"])?;
    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
#[test]
fn doctor_without_resource_reports_resources_in_canonical_order()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, _) = run(&fixture, &["doctor"])?;
    assert_eq!(code, 1);
    let positions = [
        "Manifest",
        "Config · user scope",
        "Instructions · user scope",
        "Skills · user scope",
        "Prompts · user scope",
        "Commands · user scope",
        "Rules · user scope",
        "Hooks · user scope",
    ]
    .map(
        |heading| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            Ok(stdout
                .find(heading)
                .ok_or("required test value is missing")?)
        },
    )
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    assert!(
        positions
            .windows(2)
            .map(
                |pair| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(*(pair).first().ok_or("missing fixture index 0")?
                        < *(pair).get(1).ok_or("missing fixture index 1")?)
                }
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .all(std::convert::identity),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn doctor_without_resource_fails_when_skills_drift()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    std::fs::remove_file(fixture.home().join(".cursor/skills/alpha"))?;
    let (code, stdout, _) = run(&fixture, &["doctor"])?;
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.starts_with("Manifest\n✓ 1 healthy"));
    assert!(stdout.contains("Skills · user scope"));
    assert!(stdout.contains("DRIFT alpha"));
    Ok(())
}
#[test]
fn json_doctor_without_resource_preserves_canonical_resource_order()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, _) = run(&fixture, &["doctor", "--format", "json"])?;
    let diagnostics: serde_json::Value = serde_json::from_str(&stdout)?;
    let diagnostics = diagnostics.as_array().ok_or("expected JSON array")?;
    assert_eq!(code, 1, "{stdout}");
    let resource_groups = diagnostics
        .iter()
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok((*(diagnostic)
                    .get("resource")
                    .ok_or("missing fixture index resource")?)
                .as_str()
                .ok_or("expected JSON string")?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .fold(Vec::new(), |mut groups, resource| {
            if groups.last() != Some(&resource) {
                groups.push(resource);
            }
            groups
        });
    assert_eq!(
        resource_groups,
        [
            "manifest",
            "config",
            "instructions",
            "skills",
            "prompts",
            "commands",
            "rules",
            "hooks",
        ]
    );
    Ok(())
}
#[test]
fn doctor_without_resource_stops_after_an_invalid_manifest()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        &MANIFEST.replacen("version: 1", "version: 2", 1),
    )?;
    let (code, stdout, stderr) = run(&fixture, &["doctor"])?;
    assert_eq!(code, 2);
    assert_eq!(
        stdout,
        "Manifest\n✓ 0 healthy\n\nerror manifest: version: unsupported version 2; expected 1\n"
    );
    assert!(stderr.is_empty());
    Ok(())
}
