#![cfg(test)]
mod support;
use serde_json::json;
use std::fs;
use std::os::unix::fs::symlink;
use support::Fixture;

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn fixture() -> std::result::Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\n  - { id: claude, scopes: [user, project] }\n  - { id: cursor, scopes: [user, project] }\nresources: []\n",
    )?;
    Ok(fixture)
}

#[test]
fn undeclared_mcp_fails_with_native_removal_without_mutation() -> Result {
    for (agent, path, contents, removal) in [
        (
            "codex",
            ".codex/config.toml",
            "[mcp_servers.manual]\nurl = 'https://example.test/mcp?token=private-secret'\nenabled = false\n",
            "codex mcp remove manual",
        ),
        (
            "claude",
            ".claude.json",
            r#"{"mcpServers":{"manual":{"command":"private-secret"}}}"#,
            "claude mcp remove -s user manual",
        ),
        (
            "cursor",
            ".cursor/mcp.json",
            r#"{"mcpServers":{"manual":{"url":"https://example.test/private-secret"}}}"#,
            "Cursor Settings",
        ),
    ] {
        let fixture = fixture()?;
        fixture.write_home(path, contents)?;
        let before = fixture.snapshot()?;
        let output = fixture.command(["doctor", "mcp", "--agent", agent, "--format", "json"])?;
        let stdout = String::from_utf8(output.stdout)?;
        assert_eq!(output.status.code(), Some(1), "{stdout}");
        assert!(
            stdout.contains("manual") && stdout.contains("undeclared"),
            "{stdout}"
        );
        assert!(stdout.contains(removal), "{stdout}");
        assert!(!stdout.contains("private-secret"));
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn claude_local_projects_and_project_file_are_inventoried() -> Result {
    let fixture = fixture()?;
    fixture.write_home(
        ".claude.json",
        &json!({"projects": {
            "/other project": {"mcpServers": {"local-only": {"command": "secret"}}}
        }})
        .to_string(),
    )?;
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"project-only":{"type":"http","url":"https://example.test"}}}"#,
    )?;
    let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.contains("local-only") && stdout.contains("project-only"),
        "{stdout}"
    );
    assert!(
        stdout.contains("claude mcp remove -s local local-only"),
        "{stdout}"
    );
    assert!(
        stdout.contains("claude mcp remove -s project project-only"),
        "{stdout}"
    );
    assert!(!stdout.contains("secret"));
    Ok(())
}

#[test]
fn malformed_inventory_fails_without_leaking_values() -> Result {
    for contents in [
        r#"{"mcpServers":[]}"#,
        r#"{"mcpServers":{"manual":{},"manual":{"secret":"value"}}}"#,
        r#"{"projects":{"/project":{"mcpServers":[]}}}"#,
    ] {
        let fixture = fixture()?;
        fixture.write_home(".claude.json", contents)?;
        let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
        let stdout = String::from_utf8(output.stdout)?;
        assert_eq!(output.status.code(), Some(2), "{stdout}");
        assert!(!stdout.contains("secret") && !stdout.contains("value"));
    }
    Ok(())
}

#[test]
fn empty_orphan_caches_and_settings_only_plugins_fail_read_only() -> Result {
    let fixture = fixture()?;
    for directory in [
        ".claude/plugins/cache/remem",
        ".codex/plugins/cache/semctx-stable",
        ".codex/plugins/cache/superpowers-marketplace",
        ".codex/plugins/cache/allowed/unexpected",
    ] {
        fs::create_dir_all(fixture.home().join(directory))?;
    }
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.'browser@openai-bundled']\nenabled = false\n",
    )?;
    fixture.write_home(
        ".claude/settings.json",
        r#"{"enabledPlugins":{"disabled@marketplace":false}}"#,
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["doctor", "skills", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_ne!(output.status.code(), Some(0), "{stdout}");
    for expected in [
        "cache/remem",
        "cache/semctx-stable",
        "cache/superpowers-marketplace",
        "unexpected@allowed",
        "browser@openai-bundled",
        "disabled@marketplace",
        "codex plugin remove browser@openai-bundled",
        "claude plugin uninstall disabled@marketplace",
        "codex plugin marketplace remove semctx-stable",
        "claude plugin marketplace remove remem",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }
    assert_eq!(fixture.snapshot()?, before);
    assert!(fixture.repository().is_dir());
    Ok(())
}

#[test]
fn escaping_inventory_paths_fail_without_reading_their_contents() -> Result {
    for (resource, destination) in [
        ("mcp", ".codex/config.toml"),
        ("skills", ".codex/plugins/cache"),
    ] {
        let fixture = fixture()?;
        let outside = tempfile::tempdir()?;
        fs::write(outside.path().join("private-secret"), "private-secret")?;
        let path = fixture.home().join(destination);
        fs::create_dir_all(path.parent().ok_or("parent")?)?;
        symlink(outside.path(), path)?;
        let before = fixture.snapshot()?;
        let output = fixture.command(["doctor", resource, "--format", "json"])?;
        let stdout = String::from_utf8(output.stdout)?;
        assert_eq!(output.status.code(), Some(2), "{stdout}");
        assert!(!stdout.contains("private-secret"));
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn an_agent_absent_from_manifest_still_has_its_plugins_inventoried() -> Result {
    let fixture = fixture()?;
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - { id: codex, scopes: [user] }\nresources: []\n",
    )?;
    fs::create_dir_all(fixture.home().join(".claude/plugins/cache/undeclared"))?;
    let output = fixture.command(["doctor", "skills", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("cache/undeclared"), "{stdout}");
    Ok(())
}

#[test]
fn inherited_user_plugin_settings_do_not_create_project_installations() -> Result {
    let fixture = fixture()?;
    let manifest = fs::read_to_string(fixture.home().join(".arnes.yaml"))?;
    fixture.write_home(".arnes.yaml", &format!("{manifest}\nexternal:\n  plugins:\n    - {{ agent: claude, scope: user, id: demo@marketplace }}\n"))?;
    fixture.write_home(
        ".claude/settings.json",
        r#"{"enabledPlugins":{"demo@marketplace":false}}"#,
    )?;
    let output = fixture.command([
        "doctor", "skills", "--agent", "claude", "--scope", "project", "--format", "json",
    ])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(!stdout.contains("demo@marketplace"), "{stdout}");
    fixture.write_repository(
        ".claude/settings.local.json",
        r#"{"enabledPlugins":{"demo@marketplace":false}}"#,
    )?;
    let output = fixture.command([
        "doctor", "skills", "--agent", "claude", "--scope", "project", "--format", "json",
    ])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    assert!(
        stdout.contains("claude plugin uninstall demo@marketplace --scope project"),
        "{stdout}"
    );
    Ok(())
}
