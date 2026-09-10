use crate::support::Fixture;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;
pub const MANIFEST: &str = "version: 1
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
  - id: claude-user-skills
    kind: skills
    agent: claude
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .claude/skills }
  - id: claude-project-skills
    kind: skills
    agent: claude
    scope: project
    layout: root
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .claude/skills }
  - id: cursor-user-skills
    kind: skills
    agent: cursor
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .cursor/skills }
  - id: cursor-project-skills
    kind: skills
    agent: cursor
    scope: project
    layout: root
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .cursor/skills }
  - id: codex-user-skills
    kind: skills
    agent: codex
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .agents/skills }
  - id: codex-project-skills
    kind: skills
    agent: codex
    scope: project
    layout: root
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .codex/skills }
";
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn configured_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    fixture.write_repository(
        "harness/skills/alpha/SKILL.md",
        "# Alpha\n[guide](references/guide.md)\n",
    )?;
    fixture.write_repository("harness/skills/alpha/references/guide.md", "guide\n")?;
    fixture.write_repository(".agents/skills/project-alpha/SKILL.md", "# Project Alpha\n")?;
    fixture.write_repository(
        ".agents/skills/beta/SKILL.md",
        "# Beta\n[missing](references/missing.md)\n",
    )?;
    for destination in [
        ".claude/skills/alpha",
        ".cursor/skills/alpha",
        ".agents/skills/alpha",
    ] {
        link_home(&fixture, "harness/skills/alpha", destination)?;
    }
    for destination in [".claude/skills", ".cursor/skills", ".codex/skills"] {
        link_project(&fixture, destination)?;
    }
    Ok(fixture)
}
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn link_home(
    fixture: &Fixture,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let destination = fixture.home().join(destination);
    fs::create_dir_all(destination.parent().ok_or("fixture path has no parent")?)?;
    symlink(fixture.repository().join(source), destination)?;
    Ok(())
}
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn link_home_relative(
    fixture: &Fixture,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let destination = fixture.home().join(destination);
    fs::create_dir_all(destination.parent().ok_or("fixture path has no parent")?)?;
    symlink(Path::new("../../../repository").join(source), destination)?;
    Ok(())
}
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn link_project(
    fixture: &Fixture,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let destination = fixture.repository().join(destination);
    fs::create_dir_all(destination.parent().ok_or("fixture path has no parent")?)?;
    symlink("../.agents/skills", destination)?;
    Ok(())
}
/// # Errors
/// Returns an error if the fixture command, filesystem inspection or output decoding fails.
///
/// # Panics
/// Panics if the command changes the fixture filesystem.
pub fn run(
    fixture: &Fixture,
    args: &[&str],
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let before = fixture.snapshot()?;
    let output = fixture.command(args)?;
    assert_eq!(fixture.snapshot()?, before);
    Ok((
        output
            .status
            .code()
            .ok_or("child process exited without a status code")?,
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}
