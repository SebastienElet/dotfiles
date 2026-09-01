mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use support::Fixture;

fn manifest(mcp: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user, project]\n  - id: cursor\n    scopes: [user, project]\n  - id: codex\n    scopes: [user, project]\nmcp:{mcp}\nresources: []\n"
    )
}

fn executable(fixture: &Fixture, path: &str) {
    fixture.write_repository(path, "#!/bin/sh\nexit 99\n");
    let path = fixture.repository().join(path);
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions).unwrap();
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn matching_project_registration_is_healthy_without_execution() {
    let fixture = Fixture::new();
    executable(&fixture, "bin/mcp");
    fixture.write_home(
        ".arnes.yaml",
        &manifest("\n  - { name: managed, agent: claude, scope: project, command: bin/mcp }"),
    );
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"bin/mcp"}}}"#,
    );

    let before = fixture.snapshot();
    let output = fixture.command([
        "doctor", "mcp", "--agent", "claude", "--scope", "project", "-v",
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("healthy mcp: claude project managed"));
    assert!(!fixture.home().join("executed-sentinel").exists());
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn matching_native_project_registrations_are_healthy_for_all_agents() {
    let fixture = Fixture::new();
    executable(&fixture, "bin/mcp");
    fixture.write_home(
        ".arnes.yaml",
        &manifest(
            "\n  - { name: managed, agent: claude, scope: project, command: bin/mcp }\n  - { name: managed, agent: cursor, scope: project, command: bin/mcp }\n  - { name: managed, agent: codex, scope: project, command: bin/mcp }",
        ),
    );
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"bin/mcp"}}}"#,
    );
    fixture.write_repository(
        ".cursor/mcp.json",
        r#"{"mcpServers":{"managed":{"command":"bin/mcp"}}}"#,
    );
    fixture.write_repository(
        ".codex/config.toml",
        "[mcp_servers.managed]\ncommand = \"bin/mcp\"\n",
    );

    for agent in ["claude", "cursor", "codex"] {
        let output = fixture.command([
            "doctor", "mcp", "--agent", agent, "--scope", "project", "-v",
        ]);
        assert_eq!(output.status.code(), Some(0), "{agent}");
        assert!(stdout(&output).contains(&format!("healthy mcp: {agent} project managed")));
    }
}

#[test]
fn default_doctor_checks_project_mcp_without_changing_other_scope_defaults() {
    let fixture = Fixture::new();
    executable(&fixture, "bin/mcp");
    fixture.write_home(
        ".arnes.yaml",
        &manifest("\n  - { name: managed, agent: claude, scope: project, command: bin/mcp }"),
    );
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"bin/mcp"}}}"#,
    );

    let output = fixture.command(["doctor", "-v"]);
    let stdout = stdout(&output);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("MCP"));
    assert!(stdout.contains("healthy mcp: claude project managed"));
    assert!(stdout.contains("Skills · user scope"));
}

#[test]
fn mismatches_are_redacted_and_undeclared_filters_are_unsupported() {
    let fixture = Fixture::new();
    executable(&fixture, "bin/mcp");
    fixture.write_home(".arnes.yaml", &manifest("\n  - { name: managed, agent: cursor, scope: project, command: bin/mcp, args: [expected], environment: [TOKEN] }"));
    fixture.write_repository(".cursor/mcp.json", r#"{"mcpServers":{"managed":{"command":"other","args":["actual"],"env":{"TOKEN":"actual-secret"}}}}"#);

    let output = fixture.command(["doctor", "mcp", "--agent", "cursor", "--scope", "project"]);
    let rendered = stdout(&output);
    assert_eq!(output.status.code(), Some(1));
    assert!(rendered.contains("command differs"));
    assert!(rendered.contains("ordered arguments differ"));
    assert!(rendered.contains("environment references differ"));
    assert!(!rendered.contains("actual-secret"));

    let json = fixture.command([
        "doctor", "mcp", "--agent", "cursor", "--scope", "project", "--format", "json",
    ]);
    assert!(!stdout(&json).contains("actual-secret"));

    let unsupported = fixture.command(["doctor", "mcp", "--agent", "codex"]);
    assert!(stdout(&unsupported).contains("unsupported mcp:"));
}
