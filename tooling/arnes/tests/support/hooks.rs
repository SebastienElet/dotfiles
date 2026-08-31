use crate::support::Fixture;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

pub const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [user]
hooks:
  - id: measurement
    installations:
      - { agent: claude, scope: user }
  - id: handoff
    installations:
      - { agent: claude, scope: user }
resources: []
";

pub const CURSOR_MANIFEST: &str = "version: 1
agents:
  - id: cursor
    scopes: [user]
hooks:
  - id: measurement
    installations:
      - { agent: cursor, scope: user }
resources: []
";

pub const MEASUREMENT_ONLY_MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user, project]
hooks:
  - id: measurement
    installations:
      - { agent: claude, scope: user }
resources: []
";

pub fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", MANIFEST);
    executable(&fixture, "arnes");
    executable(&fixture, "agent-handoff");
    fixture
}

pub fn installed_fixture() -> Fixture {
    let fixture = configured_fixture();
    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "claude"]);
    assert_eq!(code, 0, "{stderr}");
    fixture
}

pub fn linked_handoff_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", MANIFEST);
    executable(&fixture, "arnes");
    let target = fixture.home().join("dotfiles/tooling/agent-handoff");
    fs::create_dir_all(target.parent().unwrap()).unwrap();
    fs::write(&target, b"binary").unwrap();
    fs::set_permissions(&target, fs::Permissions::from_mode(0o700)).unwrap();
    let alias = fixture.home().join(".local/bin/agent-handoff");
    fs::create_dir_all(alias.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&target, &alias).unwrap();
    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "claude"]);
    assert_eq!(code, 0, "{stderr}");
    fixture
}

pub fn superseded_handoff_command(fixture: &Fixture) -> String {
    fixture
        .home()
        .join("dotfiles/scripts/agent_handoff")
        .to_str()
        .unwrap()
        .to_owned()
}

pub fn executable(fixture: &Fixture, name: &str) -> PathBuf {
    let path = fixture.home().join(".local/bin").join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(&path, b"binary").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
}

pub fn settings(fixture: &Fixture) -> Value {
    serde_json::from_slice(&fs::read(settings_path(fixture)).unwrap()).unwrap()
}

pub fn write_settings(fixture: &Fixture, value: &Value) {
    let path = settings_path(fixture);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, serde_json::to_vec_pretty(value).unwrap()).unwrap();
}

pub fn settings_path(fixture: &Fixture) -> PathBuf {
    fixture.home().join(".claude/settings.json")
}

pub fn run(fixture: &Fixture, args: &[&str]) -> (i32, String, String) {
    let output = fixture.command(args);
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
        String::from_utf8(output.stderr).unwrap(),
    )
}
