#![cfg(test)]
pub mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, symlink};
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [user, project]
  - id: codex
    scopes: [user, project]
skills:
  - slug: alpha
    installations:
      - { agent: claude, scope: user }
      - { agent: cursor, scope: user }
      - { agent: codex, scope: user }
resources:
  - id: claude-skills
    kind: skills
    agent: claude
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .claude/skills }
  - id: cursor-skills
    kind: skills
    agent: cursor
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .cursor/skills }
  - id: codex-skills
    kind: skills
    agent: codex
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .agents/skills }
  - id: claude-project-skills
    kind: skills
    agent: claude
    scope: project
    layout: root
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .claude/skills }
  - id: cursor-project-skills
    kind: skills
    agent: cursor
    scope: project
    layout: root
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .cursor/skills }
  - id: codex-project-skills
    kind: skills
    agent: codex
    scope: project
    layout: root
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .codex/skills }
  - id: claude-rule
    kind: rules
    agent: claude
    scope: user
    source: { root: repository, path: harness/rules/claude.md }
    destination: { root: home, path: .claude/rules/claude.md }
  - id: cursor-rule
    kind: rules
    agent: cursor
    scope: user
    source: { root: repository, path: harness/rules/cursor.mdc }
    destination: { root: home, path: .cursor/rules/cursor.mdc }
";

fn fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    fixture.write_repository("harness/skills/alpha/SKILL.md", "[guide](guide.md)\n")?;
    fixture.write_repository("harness/skills/alpha/guide.md", "guide\n")?;
    fixture.write_repository(".agents/skills/project/SKILL.md", "project\n")?;
    fixture.write_repository("harness/rules/claude.md", "claude rule\n")?;
    fixture.write_repository("harness/rules/cursor.mdc", "cursor rule\n")?;
    Ok(fixture)
}

fn sync(fixture: &Fixture, resource: &str, agent: &str, scope: &str) -> TestResult {
    let output = fixture.command(["sync", resource, "--agent", agent, "--scope", scope])?;
    assert!(
        output.status.success(),
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}

#[test]
fn creates_user_links_for_each_supported_agent_and_replays_without_rewriting() -> TestResult {
    for (resource, agent, destination, source) in [
        (
            "skills",
            "claude",
            ".claude/skills/alpha",
            "harness/skills/alpha",
        ),
        (
            "skills",
            "cursor",
            ".cursor/skills/alpha",
            "harness/skills/alpha",
        ),
        (
            "skills",
            "codex",
            ".agents/skills/alpha",
            "harness/skills/alpha",
        ),
        (
            "rules",
            "claude",
            ".claude/rules/claude.md",
            "harness/rules/claude.md",
        ),
        (
            "rules",
            "cursor",
            ".cursor/rules/cursor.mdc",
            "harness/rules/cursor.mdc",
        ),
    ] {
        let fixture = fixture()?;
        fixture.write_home("neighbor", "preserve\n")?;
        let repository_before = fixture
            .snapshot()?
            .into_iter()
            .filter(|(path, _)| path.starts_with("repository"))
            .collect::<Vec<_>>();
        sync(&fixture, resource, agent, "user")?;
        let link = fixture.home().join(destination);
        assert_eq!(
            fs::canonicalize(&link)?,
            fs::canonicalize(fixture.repository().join(source))?
        );
        let identity = fs::symlink_metadata(&link)?.ino();
        let before = fixture.snapshot()?;
        sync(&fixture, resource, agent, "user")?;
        assert_eq!(fs::symlink_metadata(&link)?.ino(), identity);
        assert_eq!(fixture.snapshot()?, before);
        assert_eq!(fs::read(fixture.home().join("neighbor"))?, b"preserve\n");
        assert_eq!(
            fixture
                .snapshot()?
                .into_iter()
                .filter(|(path, _)| path.starts_with("repository"))
                .collect::<Vec<_>>(),
            repository_before
        );
        let doctor = fixture.command([
            "doctor", resource, "--agent", agent, "--scope", "user", "--format", "json",
        ])?;
        assert!(
            doctor.status.success(),
            "{}",
            String::from_utf8_lossy(&doctor.stdout)
        );
        assert!(String::from_utf8_lossy(&doctor.stdout).contains("healthy"));
    }
    Ok(())
}

#[test]
fn creates_project_roots_for_all_agents_without_exposing_user_skills() -> TestResult {
    for (agent, destination) in [
        ("claude", ".claude/skills"),
        ("cursor", ".cursor/skills"),
        ("codex", ".codex/skills"),
    ] {
        let fixture = fixture()?;
        sync(&fixture, "skills", agent, "project")?;
        let link = fixture.repository().join(destination);
        assert_eq!(
            fs::canonicalize(&link)?,
            fs::canonicalize(fixture.repository().join(".agents/skills"))?
        );
        assert!(!link.join("alpha").exists());
        let before = fixture.snapshot()?;
        sync(&fixture, "skills", agent, "project")?;
        assert_eq!(fixture.snapshot()?, before);
        let doctor =
            fixture.command(["doctor", "skills", "--agent", agent, "--scope", "project"])?;
        assert!(
            doctor.status.success(),
            "{}",
            String::from_utf8_lossy(&doctor.stdout)
        );
    }
    Ok(())
}

#[test]
fn refuses_a_project_collection_aliasing_the_user_collection() -> TestResult {
    let fixture = fixture()?;
    fs::remove_dir_all(fixture.repository().join(".agents/skills"))?;
    symlink(
        "../harness/skills",
        fixture.repository().join(".agents/skills"),
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["sync", "skills", "--agent", "claude", "--scope", "project"])?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(fixture.snapshot()?, before);
    assert!(!fixture.repository().join(".claude/skills").exists());
    Ok(())
}

#[test]
fn refuses_a_project_skill_aliasing_a_user_skill() -> TestResult {
    let fixture = fixture()?;
    fs::remove_dir_all(fixture.repository().join(".agents/skills/project"))?;
    symlink(
        "../../harness/skills/alpha",
        fixture.repository().join(".agents/skills/project"),
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["sync", "skills", "--agent", "claude", "--scope", "project"])?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(fixture.snapshot()?, before);
    assert!(!fixture.repository().join(".claude/skills").exists());
    Ok(())
}

#[test]
fn project_skill_links_within_the_canonical_project_collection_remain_supported() -> TestResult {
    let fixture = fixture()?;
    symlink(
        "project",
        fixture.repository().join(".agents/skills/project-alias"),
    )?;
    sync(&fixture, "skills", "claude", "project")?;
    assert_eq!(
        fs::read(
            fixture
                .repository()
                .join(".claude/skills/project-alias/SKILL.md")
        )?,
        b"project\n"
    );
    let before = fixture.snapshot()?;
    sync(&fixture, "skills", "claude", "project")?;
    assert_eq!(fixture.snapshot()?, before);
    let doctor = fixture.command([
        "doctor", "skills", "--agent", "claude", "--scope", "project",
    ])?;
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    Ok(())
}

#[test]
fn project_collection_links_to_a_separate_project_directory_remain_supported() -> TestResult {
    let fixture = fixture()?;
    fs::rename(
        fixture.repository().join(".agents/skills"),
        fixture.repository().join("project-skills"),
    )?;
    symlink(
        "../project-skills",
        fixture.repository().join(".agents/skills"),
    )?;
    sync(&fixture, "skills", "claude", "project")?;
    assert_eq!(
        fs::canonicalize(fixture.repository().join(".claude/skills"))?,
        fs::canonicalize(fixture.repository().join("project-skills"))?,
    );
    let before = fixture.snapshot()?;
    sync(&fixture, "skills", "claude", "project")?;
    assert_eq!(fixture.snapshot()?, before);
    let doctor = fixture.command([
        "doctor", "skills", "--agent", "claude", "--scope", "project",
    ])?;
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    Ok(())
}

#[test]
fn refuses_existing_files_directories_wrong_and_dangling_links() -> TestResult {
    for kind in ["file", "directory", "link", "dangling", "hardlink"] {
        let fixture = fixture()?;
        let path = fixture.home().join(".claude/rules/claude.md");
        fs::create_dir_all(path.parent().ok_or("parent")?)?;
        match kind {
            "file" => fs::write(&path, "local")?,
            "directory" => fs::create_dir(&path)?,
            "link" => symlink(fixture.repository().join("harness/rules/cursor.mdc"), &path)?,
            "dangling" => symlink("missing", &path)?,
            "hardlink" => {
                fs::hard_link(fixture.repository().join("harness/rules/claude.md"), &path)?;
            }
            _ => return Err("unknown fixture".into()),
        }
        let before = fixture.snapshot()?;
        let output = fixture.command(["sync", "rules", "--agent", "claude", "--scope", "user"])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn invalid_sources_and_relative_references_leave_destinations_absent() -> TestResult {
    for missing in [
        "harness/rules/claude.md",
        "harness/skills/alpha/SKILL.md",
        "harness/skills/alpha/guide.md",
        ".agents/skills/project/SKILL.md",
    ] {
        let fixture = fixture()?;
        fs::remove_file(fixture.repository().join(missing))?;
        let resource = if missing.contains("rules") {
            "rules"
        } else {
            "skills"
        };
        let scope = if missing.starts_with(".agents") {
            "project"
        } else {
            "user"
        };
        let before = fixture.snapshot()?;
        let output = fixture.command(["sync", resource, "--agent", "claude", "--scope", scope])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn rejects_symlinked_parents_without_writes_inside_or_outside_home() -> TestResult {
    for target in ["inside", "outside"] {
        let fixture = fixture()?;
        let outside = tempfile::tempdir()?;
        fixture.write_home("inside/neighbor", "keep")?;
        symlink(
            if target == "inside" {
                fixture.home().join("inside")
            } else {
                outside.path().to_path_buf()
            },
            fixture.home().join(".claude"),
        )?;
        let before = fixture.snapshot()?;
        let output = fixture.command(["sync", "rules", "--agent", "claude", "--scope", "user"])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
        assert_eq!(fs::read_dir(outside.path())?.count(), 0);
    }
    Ok(())
}

#[test]
fn unsupported_empty_and_incomplete_selections_are_not_success() -> TestResult {
    let fixture = fixture()?;
    for args in [
        vec!["sync", "rules", "--agent", "codex", "--scope", "user"],
        vec!["sync", "rules", "--agent", "claude", "--scope", "project"],
        vec!["sync", "rules", "--agent", "claude"],
        vec!["sync", "skills", "--scope", "user"],
    ] {
        let before = fixture.snapshot()?;
        let output = fixture.command(args)?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["sync", "skills", "--agent", "claude", "--scope", "user"])?;
    assert!(!output.status.success());
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
