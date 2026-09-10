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
statuslines:
  - agent: codex
    scope: user
    items:
      - model-with-reasoning
      - current-dir
      - context-used
      - context-window-size
resources:
  - id: claude-user-instructions
    kind: instructions
    agent: claude
    scope: user
    source: { root: repository, path: harness/AGENTS.md }
    destination: { root: home, path: .claude/CLAUDE.md }
  - id: claude-user-soul
    kind: instructions
    agent: claude
    scope: user
    source: { root: repository, path: harness/SOUL.md }
    destination: { root: home, path: .claude/SOUL.md }
  - id: claude-user-preferences
    kind: instructions
    agent: claude
    scope: user
    source: { root: repository, path: harness/USER.md }
    destination: { root: home, path: .claude/USER.md }
  - id: claude-project-instructions
    kind: instructions
    agent: claude
    scope: project
    source: { root: repository, path: AGENTS.md }
    destination: { root: repository, path: CLAUDE.md }
  - id: codex-user-instructions
    kind: instructions
    agent: codex
    scope: user
    source: { root: repository, path: harness/AGENTS.md }
    destination: { root: home, path: .codex/AGENTS.md }
";
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn configured_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    fixture.write_repository("harness/AGENTS.md", "@SOUL.md\n@USER.md\nrules\n")?;
    fixture.write_repository("harness/SOUL.md", "soul\n")?;
    fixture.write_repository("harness/USER.md", "user\n")?;
    fixture.write_repository("AGENTS.md", "project rules\n")?;
    fixture.write_repository("CLAUDE.md", "See @AGENTS.md for project rules.\n")?;
    link_home(&fixture, "harness/AGENTS.md", ".claude/CLAUDE.md")?;
    link_home(&fixture, "harness/SOUL.md", ".claude/SOUL.md")?;
    link_home(&fixture, "harness/USER.md", ".claude/USER.md")?;
    fixture.write_home(".codex/AGENTS.md", "rules\nsoul\nuser\n")?;
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
    symlink(
        fs::canonicalize(fixture.repository().join(source))?,
        destination,
    )?;
    Ok(())
}
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn replace_home_link(
    fixture: &Fixture,
    source: &str,
    destination: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    fs::remove_file(fixture.home().join(destination))?;
    link_home(fixture, source, destination)?;
    Ok(())
}
/// # Errors
/// Returns an error if the fixture command, filesystem inspection or output decoding fails.
pub fn run(
    fixture: &Fixture,
    args: &[&str],
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let output = fixture.command(args)?;
    Ok((
        output
            .status
            .code()
            .ok_or("child process exited without a status code")?,
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}
/// # Errors
/// Returns an error if the fixture file or link cannot be removed.
pub fn remove(path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    fs::remove_file(path)?;
    Ok(())
}
