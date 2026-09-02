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

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", MANIFEST);
    fixture.write_repository("harness/AGENTS.md", "@SOUL.md\n@USER.md\nrules\n");
    fixture.write_repository("harness/SOUL.md", "soul\n");
    fixture.write_repository("harness/USER.md", "user\n");
    fixture.write_repository("AGENTS.md", "project rules\n");
    fixture.write_repository("CLAUDE.md", "See @AGENTS.md for project rules.\n");
    link_home(&fixture, "harness/AGENTS.md", ".claude/CLAUDE.md");
    link_home(&fixture, "harness/SOUL.md", ".claude/SOUL.md");
    link_home(&fixture, "harness/USER.md", ".claude/USER.md");
    fixture.write_home(".codex/AGENTS.md", "rules\nsoul\nuser\n");
    fixture
}

pub fn link_home(fixture: &Fixture, source: &str, destination: &str) {
    let destination = fixture.home().join(destination);
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    symlink(
        fs::canonicalize(fixture.repository().join(source)).unwrap(),
        destination,
    )
    .unwrap();
}

pub fn replace_home_link(fixture: &Fixture, source: &str, destination: &str) {
    fs::remove_file(fixture.home().join(destination)).unwrap();
    link_home(fixture, source, destination);
}

pub fn run(fixture: &Fixture, args: &[&str]) -> (i32, String, String) {
    let output = fixture.command(args);
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}

pub fn remove(path: impl AsRef<Path>) {
    fs::remove_file(path).unwrap();
}
