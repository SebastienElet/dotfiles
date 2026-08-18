#[path = "support/commands.rs"]
pub mod command_support;
#[path = "support/configured_commands.rs"]
mod configured_command_support;
pub mod support;

use command_support::{CONTENTS, command, manifest, output_tuple, prompt, run};
use configured_command_support::configured_fixture;
use std::process::Command;
use support::Fixture;

#[test]
fn claude_user_and_project_bindings_are_healthy() {
    for scope in ["user", "project"] {
        let fixture = configured_fixture();
        let (code, stdout, stderr) = run(
            &fixture,
            &[
                "doctor", "commands", "--agent", "claude", "--scope", scope, "-v",
            ],
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
    assert!(stdout.contains("error commands:"));
    assert!(stderr.is_empty());
}

#[test]
fn cursor_and_codex_are_unsupported_without_prompt_io() {
    for (agent, scope) in [
        ("cursor", "user"),
        ("cursor", "project"),
        ("codex", "user"),
        ("codex", "project"),
    ] {
        let fixture = Fixture::new();
        let prompts = prompt(
            "deploy",
            "claude",
            "project",
            "file",
            ".claude/commands/deploy.md",
        );
        let bindings = format!("      - {{ agent: {agent}, scope: {scope} }}\n");
        let commands = command("deploy", "deploy", &bindings);
        fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
        let (code, stdout, stderr) = run(
            &fixture,
            &["doctor", "commands", "--agent", agent, "--scope", scope],
        );

        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("capability · unsupported"));
        assert!(!stdout.contains("source"));
        assert!(stderr.is_empty());
    }
}

#[test]
fn filters_exclude_bindings_before_io() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt(
            "missing",
            "claude",
            "user",
            "file",
            ".claude/commands/missing.md"
        ),
        prompt(
            "selected",
            "claude",
            "project",
            "file",
            ".claude/commands/selected.md"
        ),
    );
    let commands = format!(
        "{}{}",
        command(
            "missing",
            "missing",
            "      - { agent: claude, scope: user }\n"
        ),
        command(
            "selected",
            "selected",
            "      - { agent: claude, scope: project }\n"
        ),
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    fixture.write_repository("harness/prompts/selected.md", CONTENTS);
    fixture.write_repository(".claude/commands/selected.md", CONTENTS);
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "project", "-v",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("selected · current"));
    assert!(!stdout.contains("missing"));
    assert!(stderr.is_empty());
}

#[test]
fn an_empty_filtered_selection_is_unsupported() {
    let fixture = configured_fixture();
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "cursor", "--scope", "project",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("capability · unsupported"));
    assert!(stderr.is_empty());
}

#[test]
fn diagnostics_preserve_command_then_binding_order() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt("zeta", "claude", "user", "file", ".claude/commands/zeta.md"),
        prompt(
            "alpha",
            "claude",
            "user",
            "file",
            ".claude/commands/alpha.md"
        ),
    );
    let bindings =
        "      - { agent: claude, scope: user }\n      - { agent: cursor, scope: user }\n";
    let commands = format!(
        "{}{}",
        command("zeta", "zeta", bindings),
        command("alpha", "alpha", bindings),
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    for name in ["zeta", "alpha"] {
        fixture.write_repository(format!("harness/prompts/{name}.md"), CONTENTS);
        fixture.write_home(format!(".claude/commands/{name}.md"), CONTENTS);
    }
    let (_, human, _) = run(&fixture, &["doctor", "commands", "--scope", "user", "-v"]);
    let positions = [
        "zeta · current",
        "capability · unsupported",
        "alpha · current",
    ]
    .map(|needle| human.find(needle).unwrap());
    assert!(positions[0] < positions[1] && positions[1] < positions[2]);
    let (_, json, _) = run(
        &fixture,
        &["doctor", "commands", "--scope", "user", "--format", "json"],
    );
    let diagnostics: Vec<serde_json::Value> = serde_json::from_str(&json).unwrap();

    assert!(diagnostics[0]["message"].as_str().unwrap().contains("zeta"));
    assert_eq!(diagnostics[1]["state"], "unsupported");
    assert!(
        diagnostics[2]["message"]
            .as_str()
            .unwrap()
            .contains("alpha")
    );
    assert_eq!(diagnostics[3]["state"], "unsupported");
}

#[test]
fn unmanaged_and_plugin_neighbors_are_ignored() {
    let fixture = configured_fixture();
    fixture.write_home(".claude/commands/unmanaged.md", "ignored\n");
    fixture.write_home(".claude/commands/opsx/plugin.md", "ignored\n");
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("unmanaged"));
    assert!(!stdout.contains("opsx"));
    assert!(stderr.is_empty());
}

#[test]
fn crlf_frontmatter_is_supported() {
    let fixture = configured_fixture();
    let contents = CONTENTS.replace('\n', "\r\n");
    fixture.write_repository("harness/prompts/deploy.md", &contents);
    fixture.write_repository(".claude/commands/deploy.md", &contents);
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "project", "-v",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy     deploy · current"));
    assert!(stderr.is_empty());
}
