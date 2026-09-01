#[path = "support/hooks.rs"]
pub mod hook_support;
pub mod support;

use hook_support::{
    MEASUREMENT_ONLY_MANIFEST, configured_fixture, executable, installed_fixture, run, settings,
    settings_path, write_settings,
};
use serde_json::{Value, json};
use std::fs;

fn doctor(fixture: &support::Fixture) -> (i32, String, String) {
    run(fixture, &["doctor", "hooks", "--agent", "claude", "-v"])
}

#[test]
fn setup_makes_the_declared_hooks_healthy() {
    let fixture = installed_fixture();

    let (code, stdout, stderr) = doctor(&fixture);

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
}

#[test]
fn setup_makes_the_declared_memory_hook_healthy() {
    let fixture = support::Fixture::new();
    fixture.write_home(".arnes.yaml", hook_support::MEMORY_MANIFEST);
    executable(&fixture, "arnes");
    executable(&fixture, "agent-memory");
    let (setup_code, _, setup_stderr) = run(&fixture, &["setup", "hooks", "--agent", "claude"]);
    assert_eq!(setup_code, 0, "{setup_stderr}");

    let (code, stdout, stderr) = doctor(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(
        stdout.contains("healthy hooks: claude user memory hook is installed on UserPromptSubmit"),
        "{stdout}"
    );
    assert!(stderr.is_empty());
}

#[test]
fn a_missing_configuration_file_is_drift() {
    let fixture = configured_fixture();

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 1, "{stdout}");
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is missing"),
        "{stdout}"
    );
}

#[test]
fn a_removed_event_is_drift() {
    let fixture = installed_fixture();
    let mut config = settings(&fixture);
    config["hooks"]
        .as_object_mut()
        .unwrap()
        .remove("PreToolUse");
    write_settings(&fixture, &config);

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 1);
    assert!(
        stdout.contains("claude user measurement hook is missing from PreToolUse"),
        "{stdout}"
    );
}

#[test]
fn an_unexpected_event_is_drift() {
    let fixture = installed_fixture();
    let mut config = settings(&fixture);
    let command = measurement_command(&config);
    config["hooks"]["Notification"] = json!([{"hooks":[{"type":"command","command":command}]}]);
    write_settings(&fixture, &config);

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 1);
    assert!(
        stdout.contains(
            "claude user measurement hook is installed on unexpected events Notification"
        ),
        "{stdout}"
    );
}

#[test]
fn an_installed_hook_without_a_declaration_is_drift() {
    let fixture = installed_fixture();
    fixture.write_home(".arnes.yaml", MEASUREMENT_ONLY_MANIFEST);

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 1);
    assert!(
        stdout.contains("claude user handoff hook is installed but not declared"),
        "{stdout}"
    );
}

#[test]
fn a_missing_hook_command_is_drift() {
    let fixture = installed_fixture();
    fs::remove_file(fixture.home().join(".local/bin/arnes")).unwrap();

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 1);
    assert!(
        stdout.contains("claude user measurement hook command ~/.local/bin/arnes is missing"),
        "{stdout}"
    );
}

#[test]
fn a_non_executable_hook_command_is_an_error() {
    let fixture = installed_fixture();
    fs::set_permissions(
        fixture.home().join(".local/bin/arnes"),
        <fs::Permissions as std::os::unix::fs::PermissionsExt>::from_mode(0o600),
    )
    .unwrap();

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 2);
    assert!(
        stdout.contains(
            "claude user measurement hook command ~/.local/bin/arnes is not an executable file"
        ),
        "{stdout}"
    );
}

#[test]
fn a_malformed_configuration_is_an_error() {
    let fixture = configured_fixture();
    fs::create_dir_all(settings_path(&fixture).parent().unwrap()).unwrap();
    fs::write(settings_path(&fixture), "{").unwrap();

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 2);
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is malformed"),
        "{stdout}"
    );
}

#[test]
fn an_invalid_configuration_is_an_error() {
    let fixture = configured_fixture();
    write_settings(&fixture, &json!({"hooks":{"Stop":"oops"}}));

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 2);
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is invalid"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Claude hook event must be an array"),
        "{stdout}"
    );
}

#[test]
fn project_scope_hooks_are_unsupported() {
    let fixture = installed_fixture();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "hooks", "--agent", "claude", "--scope", "project"],
    );

    assert_eq!(code, 0);
    assert!(
        stdout.contains("claude project hooks are not supported"),
        "{stdout}"
    );
}

#[test]
fn agents_without_declared_hooks_are_unsupported() {
    let fixture = installed_fixture();

    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "cursor"]);

    assert_eq!(code, 0);
    assert!(
        stdout.contains("cursor user hooks are not declared"),
        "{stdout}"
    );
}

#[test]
fn undeclared_agents_are_unsupported() {
    let fixture = installed_fixture();

    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "codex"]);

    assert_eq!(code, 0);
    assert!(
        stdout.contains(
            "unsupported hooks: codex user hook installations are not declared or supported"
        ),
        "{stdout}"
    );
}

#[test]
fn hook_diagnostics_use_the_hooks_resource() {
    let fixture = installed_fixture();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "hooks", "--agent", "claude", "--format", "json"],
    );

    assert_eq!(code, 0);
    let diagnostics: Value = serde_json::from_str(&stdout).unwrap();
    assert_eq!(diagnostics[0]["resource"], "hooks");
    assert_eq!(diagnostics[0]["state"], "healthy");
}

#[test]
fn the_hooks_doctor_is_read_only() {
    let fixture = installed_fixture();
    let before = fixture.snapshot();

    let (code, _, _) = run(&fixture, &["doctor", "hooks"]);

    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn the_default_doctor_reports_hooks() {
    let fixture = configured_fixture();

    let (code, stdout, _) = run(&fixture, &["doctor"]);

    assert_eq!(code, 1);
    assert!(stdout.contains("Hooks"), "{stdout}");
    assert!(
        stdout.contains("claude user hook configuration ~/.claude/settings.json is missing"),
        "{stdout}"
    );
}

#[test]
fn setup_restores_health_after_drift() {
    let fixture = installed_fixture();
    let mut config = settings(&fixture);
    config["hooks"].as_object_mut().unwrap().remove("Stop");
    write_settings(&fixture, &config);
    assert_eq!(doctor(&fixture).0, 1);

    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "claude"]);

    assert_eq!(code, 0, "{stderr}");
    assert_eq!(doctor(&fixture).0, 0, "{}", doctor(&fixture).1);
}

#[test]
fn direct_hook_entries_are_recognised() {
    let fixture = support::Fixture::new();
    fixture.write_home(".arnes.yaml", hook_support::CURSOR_MANIFEST);
    hook_support::executable(&fixture, "arnes");
    let (code, _, stderr) = run(&fixture, &["setup", "hooks", "--agent", "cursor"]);
    assert_eq!(code, 0, "{stderr}");

    let (code, stdout, _) = run(&fixture, &["doctor", "hooks", "--agent", "cursor", "-v"]);

    assert_eq!(code, 0, "{stdout}");
    assert!(
        stdout.contains("healthy hooks: cursor user measurement hook is installed on 12 events"),
        "{stdout}"
    );
}

#[test]
fn a_superseded_handoff_command_is_drift() {
    let fixture = hook_support::linked_handoff_fixture();
    let mut config = settings(&fixture);
    let superseded = hook_support::superseded_handoff_command(&fixture);
    config["hooks"]["Stop"]
        .as_array_mut()
        .unwrap()
        .push(json!({"hooks":[{"type":"command","command":superseded}]}));
    write_settings(&fixture, &config);

    let (code, stdout, _) = doctor(&fixture);

    assert_eq!(code, 1, "{stdout}");
    assert!(
        stdout.contains("claude user handoff hook is installed with superseded commands"),
        "{stdout}"
    );
}

fn measurement_command(config: &Value) -> String {
    config["hooks"]["PreToolUse"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap()
        .to_owned()
}
