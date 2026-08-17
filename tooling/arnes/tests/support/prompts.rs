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

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest(BASE_PROMPT));
    fixture.write_repository("harness/prompts/deploy.md", SOURCE);
    fixture.write_repository("harness/prompts/fragments/context.md", CONTEXT);
    fixture.write_repository("harness/prompts/fragments/nested/details.md", DETAILS);
    fixture.write_home(".claude/commands/deploy.md", RENDERED);
    fixture.write_repository(".claude/commands/deploy.md", SOURCE);
    fixture.write_repository(".cursor/commands/deploy.md", SOURCE);
    fixture
}

pub fn project_prompt(id: &str, agent: &str, representation: &str) -> String {
    format!(
        "  - id: {id}\n    source: {{ root: repository, path: harness/prompts/{id}.md }}\n    includes: []\n    variables: []\n    projections:\n      - agent: {agent}\n        scope: project\n        representation: {representation}\n        destination: {{ root: repository, path: .{agent}/commands/{id}.md }}\n"
    )
}

pub fn run(fixture: &Fixture, args: &[&str]) -> (i32, String, String) {
    let before = fixture.snapshot();
    let output = fixture.command(args);
    assert_eq!(fixture.snapshot(), before);
    output_tuple(output)
}

pub fn run_with_unreadable(fixture: &Fixture, path: &Path, args: &[&str]) -> (i32, String, String) {
    let before = fixture.snapshot();
    let permissions = fs::metadata(path).unwrap().permissions();
    fs::set_permissions(path, fs::Permissions::from_mode(0o000)).unwrap();
    let output = fixture.command(args);
    fs::set_permissions(path, permissions).unwrap();
    assert_eq!(fixture.snapshot(), before);
    output_tuple(output)
}

fn output_tuple(output: std::process::Output) -> (i32, String, String) {
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
