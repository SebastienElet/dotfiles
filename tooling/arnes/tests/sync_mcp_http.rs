#![cfg(test)]
mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\nmcp:\n  - name: context7\n    agent: codex\n    scope: user\n    url: https://mcp.context7.com/mcp\nresources: []\n",
    )?;
    Ok(fixture)
}

#[test]
fn rejects_invalid_transport_declarations() -> TestResult {
    for replacement in [
        "",
        "command: server\n    url: https://mcp.context7.com/mcp",
        "url: ''",
        "url: https://mcp.context7.com/mcp\n    args: [--stdio]",
        "url: https://mcp.context7.com/mcp\n    environment: [TOKEN]",
    ] {
        let fixture = fixture()?;
        let path = fixture.home().join(".arnes.yaml");
        let content =
            fs::read_to_string(&path)?.replace("url: https://mcp.context7.com/mcp", replacement);
        fixture.write_home(".arnes.yaml", &content)?;
        let before = fixture.snapshot()?;
        assert!(
            !fixture
                .command(["sync", "mcp", "--agent", "codex", "--scope", "user"])?
                .status
                .success()
        );
        assert_eq!(fixture.snapshot()?, before);
    }
    let fixture = fixture()?;
    let path = fixture.home().join(".arnes.yaml");
    fixture.write_home(
        ".arnes.yaml",
        &fs::read_to_string(path)?.replace("codex", "claude"),
    )?;
    assert!(
        !fixture
            .command(["sync", "mcp", "--agent", "claude", "--scope", "user"])?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn refuses_drift_collisions_and_malformed_native_http_without_mutation() -> TestResult {
    for fields in [
        "url = 'https://other.example/mcp'",
        "url = 42",
        "url = 'https://mcp.context7.com/mcp'\ncommand = 'private-fixture-secret'",
        "url = 'https://mcp.context7.com/mcp'\nargs = []",
        "url = 'https://mcp.context7.com/mcp'\nenv = { KEY = 'private-fixture-secret' }",
        "url = 'https://mcp.context7.com/mcp'\nhttp_headers = { Authorization = 42 }",
        "url = 'https://mcp.context7.com/mcp'\nenv_http_headers = []",
        "url = 'https://mcp.context7.com/mcp'\nbearer_token_env_var = 42",
    ] {
        let fixture = fixture()?;
        fixture.write_home(
            ".codex/config.toml",
            &format!("[mcp_servers.context7]\n{fields}\n"),
        )?;
        let before = fixture.snapshot()?;
        for operation in ["doctor", "sync"] {
            let output =
                fixture.command([operation, "mcp", "--agent", "codex", "--scope", "user"])?;
            assert!(!output.status.success());
            assert!(!String::from_utf8_lossy(&output.stdout).contains("private-fixture-secret"));
            assert!(!String::from_utf8_lossy(&output.stderr).contains("private-fixture-secret"));
        }
        assert_eq!(fixture.snapshot()?, before);
    }
    let fixture = fixture()?;
    fixture.write_repository(
        ".codex/config.toml",
        "[mcp_servers.context7]\nurl = 'https://mcp.context7.com/mcp'\n",
    )?;
    assert!(fixture.repository().join(".codex/config.toml").is_file());
    let before = fixture.snapshot()?;
    assert!(
        !fixture
            .command(["sync", "mcp", "--agent", "codex", "--scope", "user"])?
            .status
            .success()
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn changes_an_owned_stdio_entry_to_http_removing_launcher_fields() -> TestResult {
    let fixture = fixture()?;
    let path = fixture.home().join(".arnes.yaml");
    let http_manifest = fs::read_to_string(&path)?;
    fixture.write_home(
        ".arnes.yaml",
        &http_manifest.replace(
            "url: https://mcp.context7.com/mcp",
            "command: bin/server\n    args: [--stdio]\n    environment: [TOKEN]",
        ),
    )?;
    fixture.write_home("bin/server", "#!/bin/sh\nexit 0\n")?;
    fs::set_permissions(
        fixture.home().join("bin/server"),
        fs::Permissions::from_mode(0o700),
    )?;
    assert!(
        fixture
            .command(["sync", "mcp", "--agent", "codex", "--scope", "user"])?
            .status
            .success()
    );
    fixture.write_home(".arnes.yaml", &http_manifest)?;
    assert!(
        fixture
            .command(["sync", "mcp", "--agent", "codex", "--scope", "user"])?
            .status
            .success()
    );
    let document: toml::Table = toml::from_str(&fs::read_to_string(
        fixture.home().join(".codex/config.toml"),
    )?)?;
    let entry = document
        .get("mcp_servers")
        .and_then(|servers| servers.get("context7"))
        .ok_or("missing Context7")?;
    assert_eq!(
        entry.get("url").and_then(toml::Value::as_str),
        Some("https://mcp.context7.com/mcp")
    );
    for field in ["command", "args", "env", "env_vars", "cwd"] {
        assert!(entry.get(field).is_none());
    }
    Ok(())
}

#[test]
fn deploys_codex_http_without_a_launcher_and_replays() -> TestResult {
    let fixture = fixture()?;
    fixture.write_home(".codex/config.toml", "model = 'private-model'\n")?;
    let output = fixture.command(["sync", "mcp", "--agent", "codex", "--scope", "user"])?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let path = fixture.home().join(".codex/config.toml");
    let content = fs::read_to_string(&path)?;
    let document: toml::Table = toml::from_str(&content)?;
    let entry = document
        .get("mcp_servers")
        .and_then(|value| value.get("context7"))
        .ok_or("missing Context7")?;
    assert_eq!(
        entry.get("url").and_then(toml::Value::as_str),
        Some("https://mcp.context7.com/mcp")
    );
    for field in ["command", "args", "env", "env_vars"] {
        assert!(entry.get(field).is_none());
    }
    assert_eq!(
        document.get("model").and_then(toml::Value::as_str),
        Some("private-model")
    );
    let identity = fs::metadata(&path)?.ino();
    let before = fixture.snapshot()?;
    assert!(
        fixture
            .command(["sync", "mcp", "--agent", "codex", "--scope", "user"])?
            .status
            .success()
    );
    assert_eq!(fixture.snapshot()?, before);
    assert_eq!(fs::metadata(&path)?.ino(), identity);
    assert!(
        fixture
            .command(["doctor", "mcp", "--agent", "codex", "--scope", "user"])?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn accepts_native_private_authentication_without_copying_or_exposing_credentials() -> TestResult {
    for authentication in [
        "http_headers = { Authorization = 'Bearer private-fixture-secret' }",
        "bearer_token_env_var = 'CONTEXT7_API_KEY'",
        "env_http_headers = { Authorization = 'CONTEXT7_AUTHORIZATION' }",
    ] {
        let fixture = fixture()?;
        let content = format!(
            "[mcp_servers.context7]\nurl = 'https://mcp.context7.com/mcp'\n{authentication}\n"
        );
        fixture.write_home(".codex/config.toml", &content)?;
        let before = fixture.snapshot()?;
        for operation in ["sync", "doctor"] {
            let output =
                fixture.command([operation, "mcp", "--agent", "codex", "--scope", "user"])?;
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stdout)
            );
            assert!(!String::from_utf8_lossy(&output.stdout).contains("private-fixture-secret"));
            assert!(!String::from_utf8_lossy(&output.stderr).contains("private-fixture-secret"));
        }
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}
