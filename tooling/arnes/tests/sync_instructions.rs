#![cfg(test)]
pub mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, symlink};
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const MANIFEST: &str = "version: 1
agents:
  - { id: claude, scopes: [user, project] }
  - { id: codex, scopes: [user, project] }
  - { id: cursor, scopes: [user, project] }
resources:
  - id: claude-policy
    kind: instructions
    agent: claude
    scope: user
    source: { root: repository, path: harness/CLAUDE.md }
    destination: { root: home, path: .claude/CLAUDE.md }
  - id: claude-soul
    kind: instructions
    agent: claude
    scope: user
    source: { root: repository, path: harness/SOUL.md }
    destination: { root: home, path: .claude/SOUL.md }
  - id: claude-project
    kind: instructions
    agent: claude
    scope: project
    source: { root: repository, path: AGENTS.md }
    destination: { root: repository, path: CLAUDE.md }
  - id: codex-policy
    kind: instructions
    agent: codex
    scope: user
    source: { root: repository, path: harness/CLAUDE.md }
    destination: { root: home, path: .codex/AGENTS.md }
";

fn fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    fixture.write_repository("harness/CLAUDE.md", "@SOUL.md\nPolicy\n")?;
    fixture.write_repository("harness/SOUL.md", "Voice\n")?;
    fixture.write_repository("AGENTS.md", "Project\n")?;
    Ok(fixture)
}

fn sync(fixture: &Fixture, agent: &str, scope: &str) -> TestResult {
    let output = fixture.command(["sync", "instructions", "--agent", agent, "--scope", scope])?;
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let doctor = fixture.command(["doctor", "instructions", "--agent", agent, "--scope", scope])?;
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    Ok(())
}

#[test]
fn publishes_supported_representations_and_replays_without_adopting_sources() -> TestResult {
    for (agent, scope, destination) in [
        ("claude", "user", ".claude/CLAUDE.md"),
        ("claude", "project", "CLAUDE.md"),
        ("codex", "user", ".codex/AGENTS.md"),
    ] {
        let fixture = fixture()?;
        sync(&fixture, agent, scope)?;
        let path = if scope == "user" {
            fixture.home()
        } else {
            fixture.repository()
        }
        .join(destination);
        if agent == "claude" && scope == "user" {
            assert_eq!(
                fs::canonicalize(&path)?,
                fs::canonicalize(fixture.repository().join("harness/CLAUDE.md"))?
            );
        } else if agent == "codex" {
            assert_eq!(fs::read(&path)?, b"Policy\nVoice\n");
        } else {
            assert_eq!(fs::read(&path)?, b"@AGENTS.md\n");
        }
        let inode = fs::symlink_metadata(&path)?.ino();
        let before = fixture.snapshot()?;
        sync(&fixture, agent, scope)?;
        assert_eq!(fs::symlink_metadata(&path)?.ino(), inode);
        assert_eq!(fixture.snapshot()?, before);
        assert_eq!(
            fs::read(fixture.repository().join("harness/CLAUDE.md"))?,
            b"@SOUL.md\nPolicy\n"
        );
    }
    Ok(())
}

#[test]
fn updates_only_generated_files_whose_previous_publication_is_proven() -> TestResult {
    let fixture = fixture()?;
    sync(&fixture, "codex", "user")?;
    fixture.write_repository("harness/SOUL.md", "Updated voice\n")?;
    sync(&fixture, "codex", "user")?;
    assert_eq!(
        fs::read(fixture.home().join(".codex/AGENTS.md"))?,
        b"Policy\nUpdated voice\n"
    );
    fixture.write_home(".codex/AGENTS.md", "Local change\n")?;
    let before = fixture.snapshot()?;
    let output = fixture.command([
        "sync",
        "instructions",
        "--agent",
        "codex",
        "--scope",
        "user",
    ])?;
    assert!(!output.status.success());
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn conforming_existing_files_are_current_without_receipt_adoption() -> TestResult {
    let fixture = fixture()?;
    fixture.write_home(".codex/AGENTS.md", "Policy\nVoice\n")?;
    let before = fixture.snapshot()?;
    sync(&fixture, "codex", "user")?;
    assert_eq!(fixture.snapshot()?, before);
    fixture.write_repository("harness/SOUL.md", "New voice\n")?;
    let before = fixture.snapshot()?;
    let output = fixture.command([
        "sync",
        "instructions",
        "--agent",
        "codex",
        "--scope",
        "user",
    ])?;
    assert!(!output.status.success());
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn preserves_unowned_project_instructions_with_or_without_an_existing_include() -> TestResult {
    for contents in ["Local rules\n", "@AGENTS.md\nLocal rules\n"] {
        let fixture = fixture()?;
        fixture.write_repository("CLAUDE.md", contents)?;
        let before = fixture.snapshot()?;
        let output = fixture.command([
            "sync",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "project",
        ])?;
        assert_eq!(output.status.success(), contents.starts_with('@'));
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn invalid_sources_and_parent_aliases_leave_the_selection_untouched() -> TestResult {
    for kind in [
        "missing",
        "cycle",
        "outside",
        "parent",
        "wrong-link",
        "hardlink",
    ] {
        let fixture = fixture()?;
        match kind {
            "missing" => fs::remove_file(fixture.repository().join("harness/SOUL.md"))?,
            "cycle" => fixture.write_repository("harness/SOUL.md", "@CLAUDE.md\n")?,
            "outside" => fixture.write_repository("harness/CLAUDE.md", "@../../outside.md\n")?,
            "parent" => symlink(fixture.repository(), fixture.home().join(".claude"))?,
            "wrong-link" => {
                fs::create_dir(fixture.home().join(".claude"))?;
                symlink("missing", fixture.home().join(".claude/CLAUDE.md"))?;
            }
            "hardlink" => {
                fs::create_dir(fixture.home().join(".claude"))?;
                fs::hard_link(
                    fixture.repository().join("harness/CLAUDE.md"),
                    fixture.home().join(".claude/CLAUDE.md"),
                )?;
            }
            _ => return Err("invalid fixture".into()),
        }
        let before = fixture.snapshot()?;
        let output = fixture.command([
            "sync",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
        ])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn unsupported_instruction_combinations_never_mutate() -> TestResult {
    let fixture = fixture()?;
    for (agent, scope) in [
        ("cursor", "user"),
        ("cursor", "project"),
        ("codex", "project"),
    ] {
        let before = fixture.snapshot()?;
        let output =
            fixture.command(["sync", "instructions", "--agent", agent, "--scope", scope])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn colliding_instruction_declarations_are_refused_before_either_projection_is_created() -> TestResult
{
    let fixture = fixture()?;
    fixture.write_repository("README.md", "Other source\n")?;
    fixture.write_home(
        ".arnes.yaml",
        &format!(
            "{MANIFEST}  - id: duplicate-project
    kind: instructions
    agent: claude
    scope: project
    source: {{ root: repository, path: README.md }}
    destination: {{ root: repository, path: CLAUDE.md }}
"
        ),
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command([
        "sync",
        "instructions",
        "--agent",
        "claude",
        "--scope",
        "project",
    ])?;
    assert!(!output.status.success());
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
