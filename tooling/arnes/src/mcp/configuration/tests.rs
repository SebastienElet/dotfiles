use super::load;
use crate::Roots;
use crate::manifest::{Agent, Scope};
use std::fs;

fn roots() -> Result<(tempfile::TempDir, Roots), std::io::Error> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let repository = root.path().join("repository");
    fs::create_dir(&home)?;
    fs::create_dir(&repository)?;
    Ok((root, Roots::new(repository, home)))
}

fn write(path: &std::path::Path, contents: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
    fs::write(path, contents)?;
    Ok(())
}

#[test]
fn reads_native_registrations_without_retaining_literals() -> Result<(), Box<dyn std::error::Error>>
{
    let (_root, roots) = roots()?;
    write(
        &roots.home().join(".claude.json"),
        r#"{"mcpServers":{"managed":{"command":"claude-mcp","args":["--stdio"],"env":{"TOKEN":"actual-secret"}},"foreign":{"url":"https://example.invalid"}}}"#,
    )?;
    write(
        &roots.repository().join(".cursor/mcp.json"),
        r#"{"mcpServers":{"managed":{"command":"cursor-mcp","env":{"TOKEN":"${TOKEN}"}}}}"#,
    )?;
    write(
        &roots.home().join(".codex/config.toml"),
        "[mcp_servers.managed]\ncommand = \"codex-mcp\"\nenv_vars = [\"TOKEN\"]\n[mcp_servers.foreign]\nurl = \"https://example.invalid\"\n",
    )?;

    for (agent, scope, command) in [
        (Agent::Claude, Scope::User, "claude-mcp"),
        (Agent::Cursor, Scope::Project, "cursor-mcp"),
        (Agent::Codex, Scope::User, "codex-mcp"),
    ] {
        let observed = load(&roots, agent, scope, &["managed"])?.ok_or("missing fixture value")?;
        assert_eq!(
            observed
                .registrations
                .get("managed")
                .ok_or("missing managed registration")?
                .command,
            command
        );
        assert_eq!(
            observed
                .registrations
                .get("managed")
                .ok_or("missing managed registration")?
                .environment
                .len(),
            1
        );
        assert!(!observed.registrations.contains_key("foreign"));
    }
    assert!(
        !format!(
            "{:?}",
            load(&roots, Agent::Claude, Scope::User, &["managed"])?
        )
        .contains("actual-secret")
    );
    Ok(())
}

#[test]
fn malformed_and_wrong_field_types_are_errors() -> Result<(), Box<dyn std::error::Error>> {
    let (_root, roots) = roots()?;
    write(
        &roots.home().join(".claude.json"),
        r#"{"mcpServers":{"managed":{},"managed":{}}}"#,
    )?;
    assert!(
        load(&roots, Agent::Claude, Scope::User, &["managed"])
            .err()
            .ok_or("expected operation to fail")?
            .to_string()
            .contains("duplicate object key managed")
    );

    write(
        &roots.home().join(".claude.json"),
        r#"{"mcpServers":{"managed":{"command":"mcp","args":true}}}"#,
    )?;
    assert_eq!(
        load(&roots, Agent::Claude, Scope::User, &["managed"])
            .err()
            .ok_or("expected operation to fail")?
            .to_string(),
        "managed.args must be an array of strings"
    );

    write(
        &roots.home().join(".codex/config.toml"),
        "[mcp_servers.managed]\ncommand = \"actual-secret\n",
    )?;
    let error = load(&roots, Agent::Codex, Scope::User, &["managed"])
        .err()
        .ok_or("expected operation to fail")?
        .to_string();
    assert_eq!(error, "MCP configuration is malformed");
    assert!(!error.contains("actual-secret"));

    write(&roots.home().join(".cursor/mcp.json"), "[]")?;
    assert_eq!(
        load(&roots, Agent::Cursor, Scope::User, &["managed"])
            .err()
            .ok_or("expected operation to fail")?
            .to_string(),
        "MCP configuration must be an object"
    );
    Ok(())
}

#[test]
fn reads_claude_project_disabled_state() -> Result<(), Box<dyn std::error::Error>> {
    let (_root, roots) = roots()?;
    write(
        &roots.repository().join(".mcp.json"),
        r#"{"mcpServers":{"managed":{"command":"mcp"}}}"#,
    )?;
    write(
        &roots.home().join(".claude.json"),
        &format!(
            r#"{{"projects":{{"{}":{{"disabledMcpServers":["managed"]}}}}}}"#,
            roots.repository().display()
        ),
    )?;

    let observed = load(&roots, Agent::Claude, Scope::Project, &["managed"])?
        .ok_or("missing fixture value")?;
    assert_eq!(
        observed
            .registrations
            .get("managed")
            .ok_or("missing managed registration")?
            .enabled,
        Some(false)
    );
    Ok(())
}
