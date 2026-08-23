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

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", MANIFEST);
    fixture.write_repository(
        "harness/skills/agent-instructions/references/maintenance.md",
        "# Agent instructions\n",
    );
    fs::create_dir_all(fixture.repository().join("harness/rules")).unwrap();
    symlink(
        "../skills/agent-instructions/references/maintenance.md",
        fixture
            .repository()
            .join("harness/rules/agent-instructions.md"),
    )
    .unwrap();
    link_rule(&fixture, "harness/rules/agent-instructions.md");
    fixture
}

pub fn link_rule(fixture: &Fixture, source: &str) {
    let destination = fixture.home().join(".claude/rules/agent-instructions.md");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    symlink(
        fs::canonicalize(fixture.repository().join(source)).unwrap(),
        destination,
    )
    .unwrap();
}

pub fn run(fixture: &Fixture, args: &[&str]) -> (i32, String, String) {
    let output = fixture.command(args);
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
