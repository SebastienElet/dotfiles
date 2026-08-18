use crate::support::Fixture;
use std::process::Output;

pub const DESCRIPTION: &str = "Deploy safely";
pub const CONTENTS: &str = "---\ndescription: Deploy safely\n---\nDeploy now\n";

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

pub fn command(name: &str, prompt: &str, bindings: &str) -> String {
    format!(
        "  - name: {name}\n    description: {DESCRIPTION}\n    prompt: {prompt}\n    bindings:\n{bindings}"
    )
}

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    let prompts = "  - id: deploy
    source: { root: repository, path: harness/prompts/deploy.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: rendered
        destination: { root: home, path: .claude/commands/deploy.md }
      - agent: claude
        scope: project
        representation: file
        destination: { root: repository, path: .claude/commands/deploy.md }
";
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: user }\n      - { agent: claude, scope: project }\n",
    );
    fixture.write_home(".arnes.yaml", &manifest(prompts, &commands));
    fixture.write_repository("harness/prompts/deploy.md", CONTENTS);
    fixture.write_home(".claude/commands/deploy.md", CONTENTS);
    fixture.write_repository(".claude/commands/deploy.md", CONTENTS);
    fixture
}

pub fn run(fixture: &Fixture, args: &[&str]) -> (i32, String, String) {
    let before = fixture.snapshot();
    let output = fixture.command(args);
    assert_eq!(fixture.snapshot(), before);
    output_tuple(output)
}

pub fn output_tuple(output: Output) -> (i32, String, String) {
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
