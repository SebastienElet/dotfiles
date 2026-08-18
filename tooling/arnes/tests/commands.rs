#[path = "support/commands.rs"]
pub mod command_support;
pub mod support;

use command_support::{configured_fixture, output_tuple, run};
use std::process::Command;

#[test]
fn claude_user_and_project_bindings_are_healthy() {
    for scope in ["user", "project"] {
        let fixture = configured_fixture();
        let (code, stdout, stderr) = run(
            &fixture,
            &["doctor", "commands", "--agent", "claude", "--scope", scope],
        );

        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains(&format!("claude {scope} commands")));
        assert!(stdout.contains("healthy     deploy · current"));
        assert!(stderr.is_empty());
    }
}

#[test]
fn command_diagnostics_are_json_and_read_only() {
    let fixture = configured_fixture();
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "project", "--format", "json",
        ],
    );
    let diagnostics: Vec<serde_json::Value> = serde_json::from_str(&stdout).unwrap();

    assert_eq!(code, 0);
    assert_eq!(diagnostics[0]["resource"], "commands");
    assert_eq!(diagnostics[0]["state"], "healthy");
    assert!(stderr.is_empty());
}

#[test]
fn doctor_commands_routes_root_errors_to_commands() {
    let fixture = configured_fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["doctor", "commands"])
        .current_dir(fixture.repository())
        .env_clear()
        .output()
        .unwrap();
    let (code, stdout, stderr) = output_tuple(output);

    assert_eq!(code, 2);
    assert!(stdout.starts_with("error commands:"));
    assert!(stderr.is_empty());
}
