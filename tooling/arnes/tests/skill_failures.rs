#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;
use skill_support::{configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;
#[test]
fn missing_skills_and_projection_destinations_are_broken()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_file(fixture.home().join(".claude/skills/alpha"))?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("DRIFT alpha"));
    assert!(stdout.contains("actual    destination missing"));
    let fixture = configured_fixture()?;
    fs::remove_file(fixture.repository().join(".claude/skills"))?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "claude", "--scope", "project",
        ],
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("DRIFT managed skills projection"));
    assert!(stdout.contains("actual    destination missing"));
    Ok(())
}
#[test]
fn missing_user_skill_source_fails_closed() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    fs::remove_dir_all(fixture.repository().join("harness/skills"))?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("source"));
    assert!(stdout.contains("harness/skills/alpha is missing"));
    Ok(())
}
#[test]
fn broken_and_incorrect_symlinks_are_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    let link = fixture.home().join(".cursor/skills/alpha");
    fs::remove_file(&link)?;
    symlink(fixture.repository().join(".agents/skills/beta"), link)?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "cursor", "--scope", "user"],
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("wrong symlink target"));
    let fixture = configured_fixture()?;
    let link = fixture.repository().join(".codex/skills");
    fs::remove_file(&link)?;
    symlink("missing", link)?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "project"],
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("wrong symlink target"));
    Ok(())
}
#[test]
fn skill_file_must_exist_and_be_a_file() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_file(fixture.repository().join("harness/skills/alpha/SKILL.md"))?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("SKILL.md is missing"));
    let fixture = configured_fixture()?;
    let file = fixture.repository().join("harness/skills/alpha/SKILL.md");
    fs::remove_file(&file)?;
    fs::create_dir(&file)?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("SKILL.md is not a file"));
    Ok(())
}
#[test]
fn missing_relative_resources_are_broken() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_file(
        fixture
            .repository()
            .join("harness/skills/alpha/references/guide.md"),
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("local resource references/guide.md is missing"));
    let fixture = configured_fixture()?;
    fixture.write_repository(
        "harness/skills/alpha/SKILL.md",
        "[missing root resource](guide.md)\n",
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 1);
    assert!(stdout.contains("local resource guide.md is missing"));
    Ok(())
}
#[test]
fn project_source_failures_are_reported_independently()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_file(
        fixture
            .repository()
            .join(".agents/skills/project-alpha/SKILL.md"),
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "claude", "--scope", "project",
        ],
    )?;
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("broken managed claude project skill project-alpha"));
    assert!(stdout.contains("local resource references/missing.md is missing"));
    Ok(())
}
#[test]
fn unmanaged_installations_are_preserved_and_not_validated_as_owned()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(
        ".claude/skills/third-party/SKILL.md",
        "[missing](references/missing.md)\n",
    )?;
    let before = fixture.snapshot()?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("third-party · managed · enabled · healthy · unexpected"));
    assert!(!stdout.contains("local resource"));
    assert_eq!(fixture.snapshot()?, before);
    let fixture = configured_fixture()?;
    let link = fixture.home().join(".claude/skills/dangling-third-party");
    symlink("missing-third-party", link)?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("dangling-third-party · managed · unknown · broken · unexpected"));
    Ok(())
}
#[test]
fn manifest_failures_fail_closed_as_skill_errors()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    fixture.write_home(".arnes.yaml", "version: 2\n")?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "skills"])?;
    assert_eq!(code, 2);
    assert_eq!(
        stdout,
        "Skills · user scope · all agents\n✓ 0 healthy\n\nerror skills: version: unsupported version 2; expected 1\n"
    );
    assert!(stderr.is_empty());
    Ok(())
}
