#![cfg(test)]
#[path = "support/hooks.rs"]
pub mod hook_support;
pub mod support;
use hook_support::{
    MEASUREMENT_ONLY_MANIFEST, configured_fixture, executable, installed_fixture, run, settings,
    settings_path, write_settings,
};
use serde_json::{Value, json};
use std::fs;
fn doctor(
    fixture: &support::Fixture,
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    run(fixture, &["doctor", "hooks", "--agent", "claude", "-v"])
}
#[test]
fn setup_makes_the_declared_hooks_healthy() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = installed_fixture()?;
    let (code, stdout, stderr) = doctor(&fixture)?;
    assert_eq!(code, 0, "{stdout}");
    assert!(
        stdout.contains("healthy hooks: claude user measurement hook is installed on 14 events"),
        "{stdout}"
    );
    assert!(
        stdout.contains("healthy hooks: claude user handoff hook is installed on Stop"),
        "{stdout}"
    );
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn setup_makes_the_declared_memory_hook_healthy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    fixture.write_home(".arnes.yaml", hook_support::MEMORY_MANIFEST)?;
    executable(&fixture, "arnes")?;
    executable(&fixture, "agent-memory")?;
    let (setup_code, _, setup_stderr) = run(&fixture, &["setup", "hooks", "--agent", "claude"])?;
    assert_eq!(setup_code, 0, "{setup_stderr}");
    let (code, stdout, stderr) = doctor(&fixture)?;
    assert_eq!(code, 0, "{stdout}");
    assert!(
        stdout.contains("healthy hooks: claude user memory hook is installed on UserPromptSubmit"),
        "{stdout}"
    );
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn a_missing_configuration_file_is_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 1, "{stdout}");
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is missing"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn a_removed_event_is_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    let mut config = settings(&fixture)?;
    (*(config)
        .get_mut("hooks")
        .ok_or("missing fixture index hooks")?)
    .as_object_mut()
    .ok_or("expected JSON object")?
    .remove("PreToolUse");
    write_settings(&fixture, &config)?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 1);
    assert!(
        stdout.contains("claude user measurement hook is missing from PreToolUse"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn an_unexpected_event_is_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    let mut config = settings(&fixture)?;
    let command = measurement_command(&config)?;
    {
        (config)
            .get_mut("hooks")
            .ok_or("missing fixture index hooks")?
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(
                ("Notification").to_owned(),
                json ! ([{ "hooks" : [{ "type" : "command" , "command" : command }] }]),
            );
    };
    write_settings(&fixture, &config)?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 1);
    assert!(
        stdout.contains(
            "claude user measurement hook is installed on unexpected events Notification"
        ),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn an_installed_hook_without_a_declaration_is_drift()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    fixture.write_home(".arnes.yaml", MEASUREMENT_ONLY_MANIFEST)?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 1);
    assert!(
        stdout.contains("claude user handoff hook is installed but not declared"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn a_missing_hook_command_is_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    fs::remove_file(fixture.home().join(".local/bin/arnes"))?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 1);
    assert!(
        stdout.contains("claude user measurement hook command ~/.local/bin/arnes is missing"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn a_non_executable_hook_command_is_an_error()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    fs::set_permissions(
        fixture.home().join(".local/bin/arnes"),
        <fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o600),
    )?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 2);
    assert!(
        stdout.contains(
            "claude user measurement hook command ~/.local/bin/arnes is not an executable file"
        ),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn a_malformed_configuration_is_an_error() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::create_dir_all(
        settings_path(&fixture)
            .parent()
            .ok_or("fixture path has no parent")?,
    )?;
    fs::write(settings_path(&fixture), "{")?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 2);
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is malformed"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn an_invalid_configuration_is_an_error() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    write_settings(&fixture, &json ! ({ "hooks" : { "Stop" : "oops" } }))?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 2);
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is invalid"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Claude hook event must be an array"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn project_scope_hooks_are_unsupported() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "hooks", "--agent", "claude", "--scope", "project"],
    )?;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("claude project hooks are not supported"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn agents_without_declared_hooks_are_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "cursor"])?;
    assert_eq!(code, 0);
    assert!(
        stdout.contains("cursor user hooks are not declared"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn undeclared_agents_are_unsupported() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "codex"])?;
    assert_eq!(code, 0);
    assert!(
        stdout.contains(
            "unsupported hooks: codex user hook installations are not declared or supported"
        ),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn hook_diagnostics_use_the_hooks_resource() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = installed_fixture()?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "hooks", "--agent", "claude", "--format", "json"],
    )?;
    assert_eq!(code, 0);
    let diagnostics: Value = serde_json::from_str(&stdout)?;
    assert_eq!(
        *(*(diagnostics).get(0).ok_or("missing fixture index 0")?)
            .get("resource")
            .ok_or("missing fixture index resource")?,
        "hooks"
    );
    assert_eq!(
        *(*(diagnostics).get(0).ok_or("missing fixture index 0")?)
            .get("state")
            .ok_or("missing fixture index state")?,
        "healthy"
    );
    Ok(())
}
#[test]
fn the_hooks_doctor_is_read_only() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    let before = fixture.snapshot()?;
    let (code, _, _) = run(&fixture, &["doctor", "hooks"])?;
    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
#[test]
fn the_default_doctor_reports_hooks() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, _) = run(&fixture, &["doctor"])?;
    assert_eq!(code, 1);
    assert!(stdout.contains("Hooks"), "{stdout}");
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is missing"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn setup_restores_health_after_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = installed_fixture()?;
    let mut config = settings(&fixture)?;
    (*(config)
        .get_mut("hooks")
        .ok_or("missing fixture index hooks")?)
    .as_object_mut()
    .ok_or("expected JSON object")?
    .remove("Stop");
    write_settings(&fixture, &config)?;
    assert_eq!(doctor(&fixture)?.0, 1);
    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "claude"])?;
    assert_eq!(code, 0, "{stderr}");
    assert_eq!(doctor(&fixture)?.0, 0, "{}", doctor(&fixture)?.1);
    Ok(())
}
#[test]
fn direct_hook_entries_are_recognised() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    fixture.write_home(".arnes.yaml", hook_support::CURSOR_MANIFEST)?;
    hook_support::executable(&fixture, "arnes")?;
    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "cursor"])?;
    assert_eq!(code, 0, "{stderr}");
    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "cursor", "-v"])?;
    assert_eq!(code, 0, "{stdout}");
    assert!(
        stdout.contains("healthy hooks: cursor user measurement hook is installed on 12 events"),
        "{stdout}"
    );
    Ok(())
}
#[test]
fn a_superseded_handoff_command_is_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = hook_support::linked_handoff_fixture()?;
    let mut config = settings(&fixture)?;
    let superseded = hook_support::superseded_handoff_command(&fixture)?;
    (*(*(config)
        .get_mut("hooks")
        .ok_or("missing fixture index hooks")?)
    .get_mut("Stop")
    .ok_or("missing fixture index Stop")?)
    .as_array_mut()
    .ok_or("required test value is missing")?
    .push(json ! ({ "hooks" : [{ "type" : "command" , "command" : superseded }] }));
    write_settings(&fixture, &config)?;
    let (code, stdout, _) = doctor(&fixture)?;
    assert_eq!(code, 1, "{stdout}");
    assert!(
        stdout.contains("claude user handoff hook is installed with superseded commands"),
        "{stdout}"
    );
    Ok(())
}
fn measurement_command(config: &Value) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(
        (*(*(*(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("PreToolUse")
            .ok_or("missing fixture index PreToolUse")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("command")
        .ok_or("missing fixture index command")?)
        .as_str()
        .ok_or("expected JSON string")?
        .to_owned(),
    )
}
