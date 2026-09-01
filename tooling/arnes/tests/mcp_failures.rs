mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::fs::symlink;
use std::process::{Command, Output};
use support::Fixture;

fn manifest(scope: &str, command: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user, project]\nmcp:\n  - {{ name: managed, agent: claude, scope: {scope}, command: {command} }}\nresources: []\n"
    )
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

fn command_with_path(fixture: &Fixture, args: &[&str], path: &std::ffi::OsStr) -> Output {
    Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(args)
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.home())
        .env("PATH", path)
        .output()
        .unwrap()
}

#[test]
fn missing_configuration_and_registration_are_drift() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("project", "bin/mcp"));
    let missing_configuration = fixture.command(["doctor", "mcp", "--scope", "project"]);
    assert_eq!(missing_configuration.status.code(), Some(1));
    assert!(stdout(&missing_configuration).contains("configuration is missing"));

    fixture.write_repository(".mcp.json", r#"{"mcpServers":{}}"#);
    let missing_registration = fixture.command(["doctor", "mcp", "--scope", "project"]);
    assert_eq!(missing_registration.status.code(), Some(1));
    assert!(stdout(&missing_registration).contains("registration is missing"));
}

#[test]
fn same_name_in_the_other_scope_is_reported_as_collision() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("project", "bin/mcp"));
    fixture.write_repository(".mcp.json", r#"{"mcpServers":{}}"#);
    fixture.write_home(
        ".claude.json",
        r#"{"mcpServers":{"managed":{"command":"bin/mcp"}}}"#,
    );

    let output = fixture.command(["doctor", "mcp", "--scope", "project"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("registration exists in the wrong scope"));
}

#[test]
fn malformed_configuration_and_non_executable_commands_are_errors() {
    let malformed = Fixture::new();
    malformed.write_home(".arnes.yaml", &manifest("project", "bin/mcp"));
    malformed.write_repository(".mcp.json", r#"{"mcpServers":{"managed":true}}"#);
    let malformed_output = malformed.command(["doctor", "mcp", "--scope", "project"]);
    assert_eq!(malformed_output.status.code(), Some(2));
    assert!(stdout(&malformed_output).contains("managed must be an object"));

    let non_executable = Fixture::new();
    non_executable.write_home(".arnes.yaml", &manifest("project", "bin/mcp"));
    non_executable.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"bin/mcp"}}}"#,
    );
    non_executable.write_repository("bin/mcp", "never executed");
    let output = non_executable.command(["doctor", "mcp", "--scope", "project"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).contains("command is not executable"));
}

#[test]
fn bare_path_commands_are_resolved_without_execution() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "managed-mcp"));
    fixture.write_home(
        ".claude.json",
        r#"{"mcpServers":{"managed":{"command":"managed-mcp"}}}"#,
    );
    fixture.write_home("bin/managed-mcp", "#!/bin/sh\ntouch executed-sentinel\n");
    let command = fixture.home().join("bin/managed-mcp");
    let mut permissions = fs::metadata(&command).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(command, permissions).unwrap();

    let before = fixture.snapshot();
    let output = fixture.command(["doctor", "mcp", "-v"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("healthy mcp: claude user managed"));
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn path_lookup_skips_an_earlier_non_executable_candidate() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "managed-mcp"));
    fixture.write_home(
        ".claude.json",
        r#"{"mcpServers":{"managed":{"command":"managed-mcp"}}}"#,
    );
    fixture.write_home("first/managed-mcp", "not executable");
    fixture.write_home("second/managed-mcp", "#!/bin/sh\nexit 99\n");
    let executable = fixture.home().join("second/managed-mcp");
    let mut permissions = fs::metadata(&executable).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(executable, permissions).unwrap();
    let path = std::env::join_paths([fixture.home().join("first"), fixture.home().join("second")])
        .unwrap();

    let output = command_with_path(&fixture, &["doctor", "mcp", "-v"], &path);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("healthy mcp: claude user managed"));
}

#[test]
fn absolute_commands_are_checked_without_execution() {
    let fixture = Fixture::new();
    fixture.write_repository("bin/absolute-mcp", "#!/bin/sh\ntouch executed-sentinel\n");
    let command = fixture.repository().join("bin/absolute-mcp");
    let mut permissions = fs::metadata(&command).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&command, permissions).unwrap();
    fixture.write_home(
        ".arnes.yaml",
        &manifest("project", command.to_str().unwrap()),
    );
    fixture.write_repository(
        ".mcp.json",
        &format!(
            r#"{{"mcpServers":{{"managed":{{"command":"{}"}}}}}}"#,
            command.display()
        ),
    );

    let before = fixture.snapshot();
    let output = fixture.command(["doctor", "mcp", "--scope", "project", "-v"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn duplicate_scope_names_and_disabled_state_are_drift() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user, project]\nmcp:\n  - { name: managed, agent: claude, scope: user, command: mcp }\n  - { name: managed, agent: claude, scope: project, command: mcp, enabled: true }\nresources: []\n",
    );
    fixture.write_home(
        ".claude.json",
        &format!(
            r#"{{"mcpServers":{{"managed":{{"command":"mcp"}}}},"projects":{{"{}":{{"disabledMcpServers":["managed"]}}}}}}"#,
            fs::canonicalize(fixture.repository()).unwrap().display()
        ),
    );
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"mcp"}}}"#,
    );

    let output = fixture.command(["doctor", "mcp", "--scope", "project"]);
    let rendered = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(rendered.contains("registration also exists in user scope"));
    assert!(rendered.contains("enabled state differs"), "{rendered}");
}

#[test]
fn configuration_symlinks_cannot_escape_the_scope_root() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("project", "mcp"));
    let outside = tempfile::tempdir().unwrap();
    let external = outside.path().join("mcp.json");
    fs::write(
        &external,
        r#"{"mcpServers":{"managed":{"command":"secret"}}}"#,
    )
    .unwrap();
    symlink(&external, fixture.repository().join(".mcp.json")).unwrap();

    let output = fixture.command(["doctor", "mcp", "--scope", "project"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).contains("escapes its scope root"));
    assert!(!stdout(&output).contains("secret"));
}
