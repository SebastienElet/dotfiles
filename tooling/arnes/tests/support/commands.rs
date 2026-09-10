use crate::support::Fixture;
use std::process::Output;
pub const DESCRIPTION: &str = "Deploy safely";
pub const CONTENTS: &str = "---\ndescription: Deploy safely\n---\nDeploy now\n";
#[must_use]
pub fn manifest(prompts: &str, commands: &str) -> String {
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
{prompts}commands:
{commands}resources: []
"
    )
}
#[must_use]
pub fn prompt(
    id: &str,
    agent: &str,
    scope: &str,
    representation: &str,
    destination: &str,
) -> String {
    let root = if scope == "user" {
        "home"
    } else {
        "repository"
    };
    format!(
        "  - id: {id}\n    source: {{ root: repository, path: harness/prompts/{id}.md }}\n    includes: []\n    variables: []\n    projections:\n      - agent: {agent}\n        scope: {scope}\n        representation: {representation}\n        destination: {{ root: {root}, path: {destination} }}\n"
    )
}
#[must_use]
pub fn command(name: &str, prompt: &str, bindings: &str) -> String {
    format!(
        "  - name: {name}\n    description: {DESCRIPTION}\n    prompt: {prompt}\n    bindings:\n{bindings}"
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
/// Returns an error if the process has no exit code or emits invalid UTF-8.
pub fn output_tuple(
    output: Output,
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
