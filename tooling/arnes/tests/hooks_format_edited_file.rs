#![cfg(test)]
#[path = "support/hooks.rs"]
pub mod hook_support;
pub mod support;
use hook_support::{executable, run};
use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user]
  - id: codex
    scopes: [user]
hooks:
  - id: format-edited-file
    installations:
      - { agent: claude, scope: user }
      - { agent: codex, scope: user }
resources: []
";

const UNDECLARED_MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: measurement
    installations:
      - { agent: claude, scope: user }
resources: []
";

fn fixture(manifest: &str) -> Result<support::Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    fixture.write_home(".arnes.yaml", manifest)?;
    executable(&fixture, "arnes")?;
    let tool = fixture.repository().join("tooling/format-edited-file");
    fs::create_dir_all(tool.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&tool, b"tool")?;
    fs::set_permissions(&tool, fs::Permissions::from_mode(0o700))?;
    Ok(fixture)
}

fn command(fixture: &support::Fixture) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let path = fs::canonicalize(fixture.repository())?.join("tooling/format-edited-file");
    let path = path.to_str().ok_or("required test value is missing")?;
    Ok(format!("'{}'", path.replace('\'', "'\\''")))
}

fn config(
    fixture: &support::Fixture,
    relative: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&fs::read(
        fixture.home().join(relative),
    )?)?)
}

fn setup(fixture: &support::Fixture, agent: &str) -> TestResult {
    let (code, _, stderr) = run(fixture, &["setup", "hooks", "--agent", agent])?;
    assert_eq!(code, 0, "{stderr}");
    Ok(())
}

fn post_tool_use(config: &Value) -> Value {
    config
        .pointer("/hooks/PostToolUse")
        .cloned()
        .unwrap_or(Value::Null)
}

#[test]
fn installs_the_claude_hook_after_file_edits() -> TestResult {
    let fixture = fixture(MANIFEST)?;

    setup(&fixture, "claude")?;

    assert_eq!(
        post_tool_use(&config(&fixture, ".claude/settings.json")?),
        json!([{"matcher":"Edit|Write|MultiEdit","hooks":[{"type":"command","command":command(&fixture)?,"timeout":30}]}])
    );
    Ok(())
}

#[test]
fn installs_the_codex_hook_after_patches() -> TestResult {
    let fixture = fixture(MANIFEST)?;

    setup(&fixture, "codex")?;

    assert_eq!(
        post_tool_use(&config(&fixture, ".codex/hooks.json")?),
        json!([{"matcher":"apply_patch","hooks":[{"type":"command","command":command(&fixture)?,"timeout":30}]}])
    );
    Ok(())
}

#[test]
fn repeated_setup_keeps_a_single_handler() -> TestResult {
    let fixture = fixture(MANIFEST)?;

    setup(&fixture, "claude")?;
    setup(&fixture, "claude")?;

    let entries = post_tool_use(&config(&fixture, ".claude/settings.json")?);
    assert_eq!(entries.as_array().map(Vec::len), Some(1), "{entries}");
    Ok(())
}

#[test]
fn setup_removes_the_hook_once_undeclared() -> TestResult {
    let fixture = fixture(MANIFEST)?;
    setup(&fixture, "claude")?;
    fixture.write_home(".arnes.yaml", UNDECLARED_MANIFEST)?;

    setup(&fixture, "claude")?;

    let settings = config(&fixture, ".claude/settings.json")?;
    assert!(
        !settings.to_string().contains("format-edited-file"),
        "{settings}"
    );
    Ok(())
}

#[test]
fn setup_refuses_a_missing_formatter_hook_command() -> TestResult {
    let fixture = fixture(MANIFEST)?;
    fs::remove_file(fixture.repository().join("tooling/format-edited-file"))?;

    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "claude"])?;

    assert_eq!(code, 2, "{stderr}");
    assert!(stderr.contains("hook command is unavailable"), "{stderr}");
    assert!(!fixture.home().join(".claude/settings.json").exists());
    Ok(())
}

#[test]
fn the_manifest_rejects_the_hook_for_cursor() -> TestResult {
    let fixture = fixture(
        "version: 1
agents:
  - id: cursor
    scopes: [user]
hooks:
  - id: format-edited-file
    installations:
      - { agent: cursor, scope: user }
resources: []
",
    )?;

    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "cursor"])?;

    assert_eq!(code, 2, "{stderr}");
    assert!(
        stderr.contains("Cursor does not support the format-edited-file hook"),
        "{stderr}"
    );
    Ok(())
}

#[test]
fn doctor_reports_the_installed_hook_as_healthy() -> TestResult {
    let fixture = fixture(MANIFEST)?;
    setup(&fixture, "claude")?;

    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "claude", "-v"])?;

    assert_eq!(code, 0, "{stdout}");
    assert!(
        stdout.contains(
            "healthy hooks: claude user format-edited-file hook is installed on PostToolUse"
        ),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn doctor_reports_a_changed_matcher_as_drift() -> TestResult {
    let fixture = fixture(MANIFEST)?;
    setup(&fixture, "claude")?;
    let path = fixture.home().join(".claude/settings.json");
    let mut settings = config(&fixture, ".claude/settings.json")?;
    *settings
        .pointer_mut("/hooks/PostToolUse/0/matcher")
        .ok_or("missing fixture matcher")? = json!("Write");
    fs::write(&path, serde_json::to_vec_pretty(&settings)?)?;

    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "claude", "-v"])?;

    assert_eq!(code, 1, "{stdout}");
    assert!(
        stdout.contains(
            "claude user format-edited-file hook has incorrect PostToolUse matcher or execution settings"
        ),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn doctor_reports_an_undeployed_hook_as_missing() -> TestResult {
    let fixture = fixture(MANIFEST)?;
    fixture.write_home(".claude/settings.json", "{}")?;

    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "claude", "-v"])?;

    assert_eq!(code, 1, "{stdout}");
    assert!(
        stdout.contains("claude user format-edited-file hook is missing from PostToolUse"),
        "{stdout}"
    );
    Ok(())
}
