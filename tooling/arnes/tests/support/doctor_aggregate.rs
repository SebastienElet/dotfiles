use crate::support::Fixture;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: codex
    scopes: [project]
skills:
  - slug: alpha
    installations:
      - { agent: claude, scope: user }
prompts:
  - id: deploy
    source: { root: repository, path: harness/prompts/deploy.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: rendered
        destination: { root: home, path: .claude/commands/deploy.md }
commands:
  - name: deploy
    description: Deploy safely
    prompt: deploy
    bindings:
      - { agent: claude, scope: user }
hooks:
  - id: handoff
    installations:
      - { agent: claude, scope: user }
mcp:
  - { name: managed, agent: claude, scope: project, command: bin/mcp }
statuslines:
  - { agent: codex, scope: project, items: [model, current-dir] }
resources:
  - id: claude-user-instructions
    kind: instructions
    agent: claude
    scope: user
    source: { root: repository, path: harness/AGENTS.md }
    destination: { root: home, path: .claude/CLAUDE.md }
  - id: claude-user-skills
    kind: skills
    agent: claude
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .claude/skills }
  - id: aggregate-rule
    kind: rules
    agent: claude
    scope: user
    source: { root: repository, path: harness/rules/aggregate.md }
    destination: { root: home, path: .claude/rules/aggregate.md }
";

const COMMAND: &str = "---\ndescription: Deploy safely\n---\nDeploy now\n";
pub const ORDER: [&str; 10] = [
    "manifest",
    "config",
    "instructions",
    "skills",
    "prompts",
    "commands",
    "rules",
    "hooks",
    "mcp",
    "statusline",
];

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", MANIFEST);
    fixture.write_repository("harness/AGENTS.md", "agent instructions\n");
    fixture.write_repository("harness/skills/alpha/SKILL.md", "# Alpha\n");
    fixture.write_repository("harness/prompts/deploy.md", COMMAND);
    fixture.write_repository("harness/rules/aggregate.md", "rule\n");
    fixture.write_home(".claude/commands/deploy.md", COMMAND);
    link_home(&fixture, "harness/AGENTS.md", ".claude/CLAUDE.md");
    link_home(&fixture, "harness/skills/alpha", ".claude/skills/alpha");
    link_home(
        &fixture,
        "harness/rules/aggregate.md",
        ".claude/rules/aggregate.md",
    );
    fixture.write_repository("bin/mcp", "#!/bin/sh\nexit 99\n");
    set_mode(&fixture.repository().join("bin/mcp"), 0o700);
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"bin/mcp"}}}"#,
    );
    fixture.write_repository(
        ".codex/config.toml",
        "[tui]\nstatus_line = [\"model\", \"current-dir\"]\n",
    );
    fixture.write_home(".local/bin/agent-handoff", "binary\n");
    set_mode(&fixture.home().join(".local/bin/agent-handoff"), 0o700);
    let setup = fixture.command(["setup", "hooks", "--agent", "claude"]);
    assert_eq!(setup.status.code(), Some(0));
    fixture
}

fn link_home(fixture: &Fixture, source: &str, destination: &str) {
    let destination = fixture.home().join(destination);
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    symlink(
        fs::canonicalize(fixture.repository().join(source)).unwrap(),
        destination,
    )
    .unwrap();
}

pub fn set_mode(path: &std::path::Path, mode: u32) {
    let mut permissions = fs::metadata(path).unwrap().permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions).unwrap();
}
