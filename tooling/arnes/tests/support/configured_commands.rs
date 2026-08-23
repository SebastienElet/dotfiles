use crate::command_support::{CONTENTS, command, manifest};
use crate::support::Fixture;

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
