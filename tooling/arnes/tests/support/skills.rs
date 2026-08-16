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
resources:
  - id: claude-user-alpha
    kind: skills
    agent: claude
    scope: user
    source: { root: repository, path: .agents/skills/alpha }
    destination: { root: home, path: .claude/skills/alpha }
  - id: claude-project-skills
    kind: skills
    agent: claude
    scope: project
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .claude/skills }
  - id: cursor-user-alpha
    kind: skills
    agent: cursor
    scope: user
    source: { root: repository, path: .agents/skills/alpha }
    destination: { root: home, path: .cursor/skills/alpha }
  - id: cursor-project-skills
    kind: skills
    agent: cursor
    scope: project
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .cursor/skills }
  - id: codex-user-alpha
    kind: skills
    agent: codex
    scope: user
    source: { root: repository, path: .agents/skills/alpha }
    destination: { root: home, path: .agents/skills/alpha }
  - id: codex-project-skills
    kind: skills
    agent: codex
    scope: project
    source: { root: repository, path: .agents/skills }
    destination: { root: repository, path: .codex/skills }
";

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", MANIFEST);
    fixture.write_repository(
        ".agents/skills/alpha/SKILL.md",
        "# Alpha\n[guide](references/guide.md)\n",
    );
    fixture.write_repository(".agents/skills/alpha/references/guide.md", "guide\n");
    fixture.write_repository(".agents/skills/beta/SKILL.md", "# Beta\n");
    for destination in [
        ".claude/skills/alpha",
        ".cursor/skills/alpha",
        ".agents/skills/alpha",
    ] {
        link_home(&fixture, ".agents/skills/alpha", destination);
    }
    for destination in [".claude/skills", ".cursor/skills", ".codex/skills"] {
        link_project(&fixture, destination);
    }
    fixture
}

pub fn link_home(fixture: &Fixture, source: &str, destination: &str) {
    let destination = fixture.home().join(destination);
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    symlink(fixture.repository().join(source), destination).unwrap();
}

pub fn link_home_relative(fixture: &Fixture, source: &str, destination: &str) {
    let destination = fixture.home().join(destination);
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    symlink(Path::new("../../../repository").join(source), destination).unwrap();
}

pub fn link_project(fixture: &Fixture, destination: &str) {
    let destination = fixture.repository().join(destination);
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    symlink("../.agents/skills", destination).unwrap();
}

pub fn run(fixture: &Fixture, args: &[&str]) -> (i32, String, String) {
    let before = fixture.snapshot();
    let output = fixture.command(args);
    assert_eq!(fixture.snapshot(), before);
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
