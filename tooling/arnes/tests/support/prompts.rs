use crate::support::Fixture;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
pub const SOURCE: &str = "@fragments/context.md\nDeploy $environment for ${ticket}\n";
pub const CONTEXT: &str = "@nested/details.md\nContext $environment\n";
pub const DETAILS: &str = "Details ${ticket}\n";
pub const RENDERED: &str =
    "Deploy $environment for ${ticket}\nContext $environment\nDetails ${ticket}\n";
const BASE_PROMPT: &str = "  - id: deploy
    source: { root: repository, path: harness/prompts/deploy.md }
    includes: [fragments/context.md, fragments/nested/details.md]
    variables: [environment, ticket]
    projections:
      - agent: claude
        scope: user
        representation: rendered
        destination: { root: home, path: .claude/commands/deploy.md }
      - agent: claude
        scope: project
        representation: file
        destination: { root: repository, path: .claude/commands/deploy.md }
      - agent: cursor
        scope: project
        representation: file
        destination: { root: repository, path: .cursor/commands/deploy.md }
";
#[must_use]
pub fn manifest(prompts: &str) -> String {
    format!(
        "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [user, project]
  - id: codex
    scopes: [user, project]
prompts:
{prompts}resources: []
"
    )
}
/// # Errors
/// Returns an error if the fixture files, directories or symbolic links cannot be prepared.
pub fn configured_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest(BASE_PROMPT))?;
    fixture.write_repository("harness/prompts/deploy.md", SOURCE)?;
    fixture.write_repository("harness/prompts/fragments/context.md", CONTEXT)?;
    fixture.write_repository("harness/prompts/fragments/nested/details.md", DETAILS)?;
    fixture.write_home(".claude/commands/deploy.md", RENDERED)?;
    fixture.write_repository(".claude/commands/deploy.md", SOURCE)?;
    fixture.write_repository(".cursor/commands/deploy.md", SOURCE)?;
    Ok(fixture)
}
#[must_use]
pub fn project_prompt(id: &str, agent: &str, representation: &str) -> String {
    format!(
        "  - id: {id}\n    source: {{ root: repository, path: harness/prompts/{id}.md }}\n    includes: []\n    variables: []\n    projections:\n      - agent: {agent}\n        scope: project\n        representation: {representation}\n        destination: {{ root: repository, path: .{agent}/commands/{id}.md }}\n"
    )
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
    output_tuple(output)
}
/// # Errors
/// Returns an error if permissions cannot be changed, the command fails to run or output cannot be decoded.
///
/// # Panics
/// Panics if the command changes the fixture filesystem.
pub fn run_with_unreadable(
    fixture: &Fixture,
    path: &Path,
    args: &[&str],
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let before = fixture.snapshot()?;
    let permissions = fs::metadata(path)?.permissions();
    fs::set_permissions(path, fs::Permissions::from_mode(0o000))?;
    let output = fixture.command(args)?;
    fs::set_permissions(path, permissions)?;
    assert_eq!(fixture.snapshot()?, before);
    output_tuple(output)
}
fn output_tuple(
    output: std::process::Output,
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    Ok((
        output
            .status
            .code()
            .ok_or("child process exited without a status code")?,
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}
