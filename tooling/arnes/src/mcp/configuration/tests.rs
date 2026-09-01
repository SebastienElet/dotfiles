use super::load;
use crate::Roots;
use crate::manifest::{Agent, Scope};
use std::fs;

fn roots() -> (tempfile::TempDir, Roots) {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let repository = root.path().join("repository");
    fs::create_dir(&home).unwrap();
    fs::create_dir(&repository).unwrap();
    (root, Roots::new(repository, home))
}

fn write(path: &std::path::Path, contents: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

#[test]
fn reads_native_registrations_without_retaining_literals() {
    let (_root, roots) = roots();
    write(
        &roots.home().join(".claude.json"),
        r#"{"mcpServers":{"managed":{"command":"claude-mcp","args":["--stdio"],"env":{"TOKEN":"actual-secret"}},"foreign":{"url":"https://example.invalid"}}}"#,
    );
    write(
        &roots.repository().join(".cursor/mcp.json"),
        r#"{"mcpServers":{"managed":{"command":"cursor-mcp","env":{"TOKEN":"${TOKEN}"}}}}"#,
    );
    write(
        &roots.home().join(".codex/config.toml"),
        "[mcp_servers.managed]\ncommand = \"codex-mcp\"\nenv_vars = [\"TOKEN\"]\n[mcp_servers.foreign]\nurl = \"https://example.invalid\"\n",
    );

    for (agent, scope, command) in [
        (Agent::Claude, Scope::User, "claude-mcp"),
        (Agent::Cursor, Scope::Project, "cursor-mcp"),
        (Agent::Codex, Scope::User, "codex-mcp"),
    ] {
        let observed = load(&roots, agent, scope, &["managed"]).unwrap().unwrap();
        assert_eq!(observed.registrations["managed"].command, command);
        assert_eq!(observed.registrations["managed"].environment.len(), 1);
        assert!(!observed.registrations.contains_key("foreign"));
    }
    assert!(
        !format!(
            "{:?}",
            load(&roots, Agent::Claude, Scope::User, &["managed"]).unwrap()
        )
        .contains("actual-secret")
    );
}

#[test]
fn malformed_and_wrong_field_types_are_errors() {
    let (_root, roots) = roots();
    write(
        &roots.home().join(".claude.json"),
        r#"{"mcpServers":{"managed":{},"managed":{}}}"#,
    );
    assert!(
        load(&roots, Agent::Claude, Scope::User, &["managed"])
            .unwrap_err()
            .to_string()
            .contains("duplicate object key managed")
    );

    write(
        &roots.home().join(".claude.json"),
        r#"{"mcpServers":{"managed":{"command":"mcp","args":true}}}"#,
    );
    assert_eq!(
        load(&roots, Agent::Claude, Scope::User, &["managed"])
            .unwrap_err()
            .to_string(),
        "managed.args must be an array of strings"
    );

    write(
        &roots.home().join(".codex/config.toml"),
        "[mcp_servers.managed]\ncommand = \"actual-secret\n",
    );
    let error = load(&roots, Agent::Codex, Scope::User, &["managed"])
        .unwrap_err()
        .to_string();
    assert_eq!(error, "MCP configuration is malformed");
    assert!(!error.contains("actual-secret"));

    write(&roots.home().join(".cursor/mcp.json"), "[]");
    assert_eq!(
        load(&roots, Agent::Cursor, Scope::User, &["managed"])
            .unwrap_err()
            .to_string(),
        "MCP configuration must be an object"
    );
}

#[test]
fn reads_claude_project_disabled_state() {
    let (_root, roots) = roots();
    write(
        &roots.repository().join(".mcp.json"),
        r#"{"mcpServers":{"managed":{"command":"mcp"}}}"#,
    );
    write(
        &roots.home().join(".claude.json"),
        &format!(
            r#"{{"projects":{{"{}":{{"disabledMcpServers":["managed"]}}}}}}"#,
            roots.repository().display()
        ),
    );

    let observed = load(&roots, Agent::Claude, Scope::Project, &["managed"])
        .unwrap()
        .unwrap();
    assert_eq!(observed.registrations["managed"].enabled, Some(false));
}
