use crate::support::Fixture;
use std::fs;
use std::os::unix::fs::symlink;
pub const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [user, project]
  - id: codex
    scopes: [user, project]
resources:
  - id: agent-instructions
    kind: rules
    agent: claude
    scope: user
    source: { root: repository, path: harness/rules/agent-instructions.md }
    destination: { root: home, path: .claude/rules/agent-instructions.md }
";
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn configured_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    fixture.write_repository(
        "harness/skills/agent-instructions/references/maintenance.md",
        "# Agent instructions\n",
    )?;
    fs::create_dir_all(fixture.repository().join("harness/rules"))?;
    symlink(
        "../skills/agent-instructions/references/maintenance.md",
        fixture
            .repository()
            .join("harness/rules/agent-instructions.md"),
    )?;
    link_rule(&fixture, "harness/rules/agent-instructions.md")?;
    Ok(fixture)
}
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn link_rule(
    fixture: &Fixture,
    source: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::create_dir_all(destination.parent().ok_or("fixture path has no parent")?)?;
    symlink(
        fs::canonicalize(fixture.repository().join(source))?,
        destination,
    )?;
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
