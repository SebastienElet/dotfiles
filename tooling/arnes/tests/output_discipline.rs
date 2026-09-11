#![cfg(test)]
#[path = "support/mod.rs"]
mod support;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::Command;
use support::Fixture;
type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;
const SOURCE: &str = "harness/skills/output-discipline/SKILL.md";

#[test]
fn absent_marker_is_silent_even_with_broken_deployment() -> Result {
    let fixture = Fixture::new()?;
    symlink("/missing/deployment", fixture.home().join(".arnes.yaml"))?;
    let output = fixture.command(["output-discipline"])?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
    Ok(())
}

#[test]
fn opt_in_loads_canonical_source_outside_repository_and_strips_frontmatter() -> Result {
    for (source, body) in [
        ("---\nname: hidden\n---\nVisible\n", "Visible"),
        ("--- \r\nname: hidden\r\n---\t\r\nVisible\r\n", "Visible"),
        ("Visible", "Visible"),
        ("---\nUnclosed", "---\nUnclosed"),
    ] {
        let fixture = Fixture::new()?;
        fixture.write_repository(SOURCE, source)?;
        deploy_manifest(&fixture)?;
        fixture.write_home(".claude/.output-discipline-always", "")?;
        let output = fixture.command_from(fixture.home(), ["output-discipline"])?;
        assert!(output.status.success());
        assert!(output.stderr.is_empty());
        let text = String::from_utf8(output.stdout)?;
        assert!(text.starts_with("OUTPUT DISCIPLINE ACTIVE"));
        assert!(text.ends_with(&format!("{body}\n")), "{text}");
        assert!(!text.contains("name: hidden"));
    }
    Ok(())
}

#[test]
fn custom_config_directory_controls_opt_in() -> Result {
    let fixture = Fixture::new()?;
    fixture.write_repository(SOURCE, "Visible")?;
    deploy_manifest(&fixture)?;
    fixture.write_home(".claude/.output-discipline-always", "")?;
    let custom = fixture.home().join("custom");
    for enabled in [false, true] {
        if enabled {
            fixture.write_home("custom/.output-discipline-always", "")?;
        }
        let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
            .arg("output-discipline")
            .env_clear()
            .env("HOME", fixture.home())
            .env("CLAUDE_CONFIG_DIR", &custom)
            .current_dir(fixture.repository())
            .output()?;
        assert!(output.status.success());
        assert_eq!(!output.stdout.is_empty(), enabled);
        assert!(output.stderr.is_empty());
    }
    Ok(())
}

#[test]
fn missing_or_unreadable_source_warns_without_blocking_session() -> Result {
    for directory in [false, true] {
        let fixture = Fixture::new()?;
        fixture.write_home(".claude/.output-discipline-always", "")?;
        deploy_manifest(&fixture)?;
        if directory {
            fs::create_dir_all(fixture.repository().join(SOURCE))?;
        }
        let output = fixture.command(["output-discipline"])?;
        assert!(output.status.success());
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr)?.contains(SOURCE));
    }
    Ok(())
}

#[test]
fn setup_reconciles_loader_preserving_unrelated_hooks_and_doctor_detects_matcher_drift() -> Result {
    for (agent, path) in [
        ("claude", ".claude/settings.json"),
        ("codex", ".codex/hooks.json"),
    ] {
        let fixture = Fixture::new()?;
        let manifest = format!(
            "version: 1\nagents:\n  - id: {agent}\n    scopes: [user]\nhooks:\n  - id: output-discipline\n    installations:\n      - {{ agent: {agent}, scope: user }}\nresources: []\n"
        );
        fixture.write_home(".arnes.yaml", &manifest)?;
        fixture.write_home(".local/bin/arnes", "binary")?;
        fs::set_permissions(
            fixture.home().join(".local/bin/arnes"),
            fs::Permissions::from_mode(0o700),
        )?;
        fixture.write_home(path, r#"{"unrelated":true,"hooks":{"SessionStart":[{"matcher":"startup","hooks":[{"type":"command","command":"echo unrelated"}]}]}}"#)?;
        let output = fixture.command(["setup", "hooks", "--agent", agent])?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let before = fixture.snapshot()?;
        assert!(
            fixture
                .command(["setup", "hooks", "--agent", agent])?
                .status
                .success()
        );
        assert_eq!(before, fixture.snapshot()?);
        let config_path = fixture.home().join(path);
        let mut config: serde_json::Value = serde_json::from_slice(&fs::read(&config_path)?)?;
        assert_eq!(config.pointer("/unrelated").ok_or("unrelated field")?, true);
        assert_eq!(
            config
                .pointer("/hooks/SessionStart/0/hooks/0/command")
                .ok_or("unrelated command")?,
            "echo unrelated"
        );
        assert_eq!(
            config
                .pointer("/hooks/SessionStart/1/matcher")
                .ok_or("matcher")?,
            "startup|resume|clear|compact"
        );
        assert_eq!(
            config
                .pointer("/hooks/SessionStart/1/hooks/0/timeout")
                .ok_or("timeout")?,
            30
        );
        assert!(
            fixture
                .command(["doctor", "hooks", "--agent", agent])?
                .status
                .success()
        );
        assert_execution_drift(&fixture, agent, &config_path, &mut config)?;
        fixture.write_home(
            ".arnes.yaml",
            &format!("version: 1\nagents:\n  - id: {agent}\n    scopes: [user]\nresources: []\n"),
        )?;
        assert!(
            fixture
                .command(["setup", "hooks", "--agent", agent])?
                .status
                .success()
        );
        let config: serde_json::Value = serde_json::from_slice(&fs::read(config_path)?)?;
        assert_eq!(
            config
                .pointer("/hooks/SessionStart")
                .ok_or("event")?
                .as_array()
                .ok_or("groups")?
                .len(),
            1
        );
    }
    Ok(())
}

#[test]
fn cursor_declaration_is_rejected_without_writing_config() -> Result {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", "version: 1\nagents:\n  - id: cursor\n    scopes: [user]\nhooks:\n  - id: output-discipline\n    installations:\n      - { agent: cursor, scope: user }\nresources: []\n")?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["setup", "hooks", "--agent", "cursor"])?;
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)?
            .contains("Cursor does not support the output-discipline hook")
    );
    assert_eq!(before, fixture.snapshot()?);
    Ok(())
}

#[test]
fn each_invocation_reloads_when_marker_is_present() -> Result {
    let fixture = Fixture::new()?;
    fixture.write_repository(SOURCE, "Visible")?;
    deploy_manifest(&fixture)?;
    fixture.write_home(".claude/.output-discipline-always", "")?;
    let first = fixture.command(["output-discipline"])?;
    let second = fixture.command(["output-discipline"])?;
    assert!(first.status.success());
    assert_eq!(first.stdout, second.stdout);
    assert!(!second.stdout.is_empty());
    Ok(())
}

#[test]
fn broken_deployment_warns_only_when_opted_in() -> Result {
    let fixture = Fixture::new()?;
    fixture.write_home(".claude/.output-discipline-always", "")?;
    symlink("/missing/deployment", fixture.home().join(".arnes.yaml"))?;
    let output = fixture.command(["output-discipline"])?;
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("symlink could not be resolved"));
    Ok(())
}

fn assert_execution_drift(
    fixture: &Fixture,
    agent: &str,
    path: &std::path::Path,
    config: &mut serde_json::Value,
) -> Result {
    let original = config.clone();
    for (pointer, replacement) in [
        (
            "/hooks/SessionStart/1/matcher",
            serde_json::json!("startup"),
        ),
        (
            "/hooks/SessionStart/1/hooks/0/timeout",
            serde_json::json!(1),
        ),
        (
            "/hooks/SessionStart/1/hooks/0",
            serde_json::json!({
                "type":"command", "command":original.pointer("/hooks/SessionStart/1/hooks/0/command"),
                "timeout":30, "async":true
            }),
        ),
        (
            "/hooks/SessionStart",
            serde_json::json!([
                original.pointer("/hooks/SessionStart/1"),
                original.pointer("/hooks/SessionStart/1")
            ]),
        ),
    ] {
        *config = original.clone();
        *config.pointer_mut(pointer).ok_or("execution field")? = replacement;
        fs::write(path, serde_json::to_vec(config)?)?;
        assert!(
            !fixture
                .command(["doctor", "hooks", "--agent", agent])?
                .status
                .success()
        );
    }
    *config = original;
    fs::write(path, serde_json::to_vec(config)?)?;
    Ok(())
}

#[test]
fn missing_or_ordinary_manifest_never_loads_current_project_skill() -> Result {
    for ordinary in [false, true] {
        let fixture = Fixture::new()?;
        fixture.write_home(".claude/.output-discipline-always", "")?;
        fixture.write_repository(SOURCE, "WRONG PROJECT SENTINEL")?;
        if ordinary {
            fixture.write_home(".arnes.yaml", "version: 1\n")?;
        }
        let output = fixture.command(["output-discipline"])?;
        assert!(output.status.success());
        assert!(
            output.stdout.is_empty(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(!output.stderr.is_empty());
    }
    Ok(())
}

fn deploy_manifest(fixture: &Fixture) -> Result {
    fixture.write_repository("home/.arnes.yaml", "version: 1\n")?;
    symlink(
        fixture.repository().join("home/.arnes.yaml"),
        fixture.home().join(".arnes.yaml"),
    )?;
    Ok(())
}
