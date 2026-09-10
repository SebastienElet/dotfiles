#![cfg(test)]
#[path = "support/instructions.rs"]
pub mod instruction_support;
pub mod support;
use instruction_support::{configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
use std::process::Command;
use support::Fixture;
#[test]
fn user_scope_is_default_for_supported_and_unsupported_projections()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "instructions", "-v"])?;
    assert_eq!(code, 0);
    assert_eq!(stdout.matches("healthy instructions:").count(), 4);
    assert_eq!(stdout.matches("unsupported instructions:").count(), 1);
    for expected in [
        "healthy instructions: claude user instructions claude-user-instructions",
        "healthy instructions: claude user instructions claude-user-soul",
        "healthy instructions: claude user instructions claude-user-preferences",
        "unsupported instructions: cursor user instruction projection",
        "healthy instructions: codex user instructions codex-user-instructions",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn user_instruction_targets_from_the_deployed_checkout_are_healthy_from_a_worktree()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository("home/.arnes.yaml", instruction_support::MANIFEST)?;
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
        [
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
        ],
    )?;
    let stdout = String::from_utf8(output.stdout)?;
    assert!(output.status.success(), "{stdout}");
    assert!(stdout.contains("✓ 3 healthy"), "{stdout}");
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
fn agent_and_scope_filters_isolate_projections()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".codex/AGENTS.md", "stale\n")?;
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
    )?;
    assert_eq!(code, 0);
    assert_eq!(stdout.matches("healthy instructions:").count(), 3);
    assert!(!stdout.contains("codex"));
    let (code, stdout, _) = run(&fixture, &["doctor", "instructions", "--agent", "cursor"])?;
    assert_eq!(code, 0);
    assert_eq!(stdout.matches("unsupported instructions:").count(), 1);
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
    )?;
    assert_eq!(code, 0);
    assert_eq!(stdout.matches("unsupported instructions:").count(), 1);
    assert!(stdout.contains("unsupported instructions: codex project"));
    Ok(())
}
#[test]
fn undeclared_filtered_combinations_are_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    )?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "instructions", "--agent", "codex"])?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        "Instructions · user scope · codex agent\n✓ 0 healthy\n! 1 unsupported (non-blocking)\n\nunsupported instructions: codex user instruction projection is not declared or supported\n"
    );
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn codex_generated_content_must_match_declared_composition()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".codex/AGENTS.md", "rules\nsoul\nold user\n")?;
    let before = fixture.snapshot()?;
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
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("generated file ~/.codex/AGENTS.md is stale"));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
#[test]
fn repository_agents_file_has_precedence_for_claude_project()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository("CLAUDE.md", "See @harness/AGENTS.md instead.\n")?;
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
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("does not include source AGENTS.md"));
    Ok(())
}
#[test]
fn instructions_doctor_is_read_only() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let before = fixture.snapshot()?;
    let (code, _, _) = run(&fixture, &["doctor", "instructions"])?;
    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
#[test]
fn default_doctor_reuses_filtered_instruction_diagnostics()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_file(fixture.home().join(".codex/AGENTS.md"))?;
    let (_, direct, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "codex",
            "--scope",
            "user",
            "--format",
            "json",
        ],
    )?;
    let (_, aggregate, _) = run(
        &fixture,
        &[
            "doctor", "--agent", "codex", "--scope", "user", "--format", "json",
        ],
    )?;
    let direct: Vec<serde_json::Value> = serde_json::from_str(&direct)?;
    let aggregate: Vec<serde_json::Value> = serde_json::from_str(&aggregate)?;
    let aggregate = aggregate
        .into_iter()
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(diagnostic)
                        .get("resource")
                        .ok_or("missing fixture index resource")?
                        == "instructions"
                };
                Ok((include, diagnostic))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();
    assert_eq!(aggregate, direct);
    Ok(())
}
