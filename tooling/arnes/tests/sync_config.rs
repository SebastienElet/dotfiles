#![cfg(test)]
mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use support::Fixture;

#[test]
fn current_repository_sources_and_locks_are_reserved_before_native_config_open() -> TestResult {
    for source in [".claude/settings.json", ".claude/.settings.json.lock"] {
        let fixture = Fixture::new()?;
        let manifest = format!(
            "version: 1\nagents:\n  - id: claude\n    scopes: [user, project]\n    user_config: {{ model: expected }}\nresources: []\nprompts:\n  - id: canonical\n    source: {{ root: repository, path: {source} }}\n    includes: []\n    variables: []\n    projections: []\n"
        );
        fixture.write_repository("home/.arnes.yaml", &manifest)?;
        symlink(
            fixture.repository().join("home/.arnes.yaml"),
            fixture.home().join(".arnes.yaml"),
        )?;
        fixture.write_home(
            source,
            if source.ends_with("settings.json") {
                r#"{"model":"local"}"#
            } else {
                "Canonical lock source\n"
            },
        )?;
        if source == ".claude/.settings.json.lock" {
            fixture.write_home(".claude/settings.json", r#"{"model":"expected"}"#)?;
        }
        let before = fixture.snapshot()?;
        let output = fixture.command_from(
            fixture.home(),
            ["sync", "config", "--agent", "claude", "--scope", "user"],
        )?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

#[test]
fn an_unavailable_unselected_mcp_command_does_not_block_user_config() -> TestResult {
    let fixture = Fixture::new()?;
    let mcp = "\nmcp:\n  - name: unavailable\n    agent: claude\n    scope: user\n    command: missing-unselected-server\n    args: []\n    environment: []\n";
    fixture.write_home(".arnes.yaml", &(manifest("claude") + mcp))?;
    assert_eq!(sync(&fixture, "claude", "user")?.status.code(), Some(0));
    assert!(
        fixture
            .command(["doctor", "config", "--agent", "claude", "--scope", "user"])?
            .status
            .success()
    );
    let before = fixture.snapshot()?;
    assert_eq!(
        fixture
            .command(["sync", "mcp", "--agent", "claude", "--scope", "user"])?
            .status
            .code(),
        Some(1)
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

fn manifest(agent: &str) -> String {
    let values = match agent {
        "claude" => "model: native-model, effort: high, auto_compact_window: 300000",
        "cursor" => "model: native-model, max_mode: false",
        "codex" => {
            "model: native-model, effort: high, context_window: 512000, auto_compact_window: 300000"
        }
        _ => "model: native-model",
    };
    format!(
        "version: 1\nagents:\n  - id: {agent}\n    scopes: [user, project]\n    user_config: {{ {values} }}\nresources: []\n"
    )
}

fn sync(
    fixture: &Fixture,
    agent: &str,
    scope: &str,
) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
    fixture.command([
        "sync", "config", "--agent", agent, "--scope", scope, "--format", "json",
    ])
}

#[test]
fn synchronizes_json_defaults_without_changing_unknown_numbers_or_sections() -> TestResult {
    for (agent, path, original) in [
        (
            "claude",
            ".claude/settings.json",
            r#"{"model":"private-old","hooks":{"Stop":[{"command":"private-hook"}]},"mcpServers":{"local":{"command":"private-mcp"}},"statusLine":{"command":"private-status"},"future":{"large":9007199254740993,"frac":0.10000000000000000001,"neg":-0,"exp":1.000e+10}}"#,
        ),
        (
            "cursor",
            ".cursor/cli-config.json",
            r#"{"model":{"modelId":"private-old","displayName":"keep","counter":9007199254740993},"maxMode":true,"future":{"large":9007199254740993,"frac":0.10000000000000000001,"neg":-0,"exp":1.000e+10}}"#,
        ),
    ] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", &manifest(agent))?;
        fixture.write_home(path, original)?;
        fixture.write_repository(path, original)?;
        let output = sync(&fixture, agent, "user")?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let report = String::from_utf8(output.stdout)?;
        assert!(!report.contains("private-"));
        let actual = fs::read_to_string(fixture.home().join(path))?;
        for token in [
            "9007199254740993",
            "0.10000000000000000001",
            "-0",
            "1.000e+10",
        ] {
            assert!(actual.contains(token), "lost {token}: {actual}");
        }
        assert_eq!(
            fs::read_to_string(fixture.repository().join(path))?,
            original
        );
        if agent == "claude" {
            for unchanged in ["private-hook", "private-mcp", "private-status"] {
                assert!(actual.contains(unchanged));
            }
        } else {
            assert!(actual.contains("keep"));
        }
        let identity = fs::metadata(fixture.home().join(path))?.ino();
        let before = fixture.snapshot()?;
        let output = sync(&fixture, agent, "user")?;
        assert_eq!(output.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&output.stdout).contains("current"));
        assert_eq!(fs::metadata(fixture.home().join(path))?.ino(), identity);
        assert_eq!(fixture.snapshot()?, before);
        let doctor = fixture.command(["doctor", "config", "--agent", agent, "--scope", "user"])?;
        assert_eq!(
            doctor.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&doctor.stdout)
        );
    }
    Ok(())
}

#[test]
fn synchronizes_codex_defaults_preserving_tui_mcp_and_nan_and_replays() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest("codex"))?;
    fixture.write_home(".codex/config.toml", "model = 'private-old'\n[unknown]\nvalue = nan\nlarge = 9007199254740993\n[tui]\nstatus_line = ['model']\n[mcp_servers.local]\ncommand = 'private-mcp'\n")?;
    let output = sync(&fixture, "codex", "user")?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let path = fixture.home().join(".codex/config.toml");
    let value: toml::Value = toml::from_str(&fs::read_to_string(&path)?)?;
    assert_eq!(
        value.get("model").and_then(toml::Value::as_str),
        Some("native-model")
    );
    assert!(
        value
            .get("unknown")
            .and_then(|v| v.get("value"))
            .and_then(toml::Value::as_float)
            .ok_or("nan")?
            .is_nan()
    );
    assert_eq!(
        value
            .get("unknown")
            .and_then(|v| v.get("large"))
            .and_then(toml::Value::as_integer),
        Some(9_007_199_254_740_993)
    );
    assert_eq!(
        value
            .get("mcp_servers")
            .and_then(|v| v.get("local"))
            .and_then(|v| v.get("command"))
            .and_then(toml::Value::as_str),
        Some("private-mcp")
    );
    let before = fixture.snapshot()?;
    let output = sync(&fixture, "codex", "user")?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("current"));
    assert_eq!(fixture.snapshot()?, before);
    assert!(
        fixture
            .command(["doctor", "config", "--agent", "codex", "--scope", "user"])?
            .status
            .success()
    );
    Ok(())
}

#[test]
fn creates_missing_native_configurations_for_declared_user_defaults() -> TestResult {
    for agent in ["claude", "cursor", "codex"] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", &manifest(agent))?;
        let output = sync(&fixture, agent, "user")?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            fixture
                .command(["doctor", "config", "--agent", agent, "--scope", "user"])?
                .status
                .success()
        );
    }
    Ok(())
}

#[test]
fn project_and_empty_selections_are_explicit_without_mutation() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest("claude"))?;
    let before = fixture.snapshot()?;
    assert_eq!(sync(&fixture, "claude", "project")?.status.code(), Some(1));
    assert_eq!(fixture.snapshot()?, before);
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    )?;
    let before = fixture.snapshot()?;
    let output = sync(&fixture, "claude", "user")?;
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("empty"));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn malformed_and_incompatible_json_is_preserved_without_exposing_values() -> TestResult {
    for original in [
        "[",
        "[]",
        r#"{"model":42}"#,
        r#"{"model":"private","model":"duplicate"}"#,
        r#"{"model":{"modelId":"wrong-shape"}}"#,
    ] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", &manifest("claude"))?;
        fixture.write_home(".claude/settings.json", original)?;
        let output = sync(&fixture, "claude", "user")?;
        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(fixture.home().join(".claude/settings.json"))?,
            original
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("private"));
    }
    Ok(())
}

#[test]
fn native_links_and_write_failures_preserve_canonical_and_existing_files() -> TestResult {
    for kind in ["link", "hardlink", "parent-link", "write-failure"] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", &manifest("claude"))?;
        fixture.write_repository("home/.claude/settings.json", "{\"model\":\"private\"}")?;
        let source = fixture.repository().join("home/.claude/settings.json");
        let destination = fixture.home().join(".claude/settings.json");
        fs::create_dir(fixture.home().join(".claude"))?;
        match kind {
            "link" => symlink(&source, &destination)?,
            "hardlink" => fs::hard_link(&source, &destination)?,
            "parent-link" => {
                fs::remove_dir(fixture.home().join(".claude"))?;
                symlink(
                    fixture.repository().join("home/.claude"),
                    fixture.home().join(".claude"),
                )?;
            }
            "write-failure" => {
                fs::write(&destination, "{\"model\":\"private\"}")?;
                fs::set_permissions(
                    fixture.home().join(".claude"),
                    fs::Permissions::from_mode(0o500),
                )?;
            }
            _ => return Err("unknown fixture".into()),
        }
        let output = sync(&fixture, "claude", "user")?;
        if kind == "write-failure" {
            fs::set_permissions(
                fixture.home().join(".claude"),
                fs::Permissions::from_mode(0o700),
            )?;
        }
        assert!(!output.status.success());
        assert_eq!(fs::read_to_string(source)?, "{\"model\":\"private\"}");
        assert_eq!(fs::read_to_string(destination)?, "{\"model\":\"private\"}");
    }
    Ok(())
}
