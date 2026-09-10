#![cfg(test)]
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
fn claude_user_and_project_bindings_are_healthy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for scope in ["user", "project"] {
        let fixture = configured_fixture()?;
        let (code, stdout, stderr) = run(
            &fixture,
            &[
                "doctor", "commands", "--agent", "claude", "--scope", scope, "-v",
            ],
        )?;
        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains(&format!("claude {scope} commands")));
        assert!(stdout.contains("healthy     deploy · current"));
        assert!(stderr.is_empty());
    };
    Ok(())
}
#[test]
fn command_diagnostics_are_json_and_read_only()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "project", "--format", "json",
        ],
    )?;
    let diagnostics: Vec<serde_json::Value> = serde_json::from_str(&stdout)?;
    assert_eq!(code, 0);
    assert_eq!(
        *(*(diagnostics).first().ok_or("missing fixture index 0")?)
            .get("resource")
            .ok_or("missing fixture index resource")?,
        "commands"
    );
    assert_eq!(
        *(*(diagnostics).first().ok_or("missing fixture index 0")?)
            .get("state")
            .ok_or("missing fixture index state")?,
        "healthy"
    );
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn doctor_commands_routes_root_errors_to_commands()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["doctor", "commands"])
        .current_dir(fixture.repository())
        .env_clear()
        .output()?;
    let (code, stdout, stderr) = output_tuple(output)?;
    assert_eq!(code, 2);
    assert!(stdout.contains("error commands:"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn cursor_and_codex_are_unsupported_without_prompt_io()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (agent, scope) in [
        ("cursor", "user"),
        ("cursor", "project"),
        ("codex", "user"),
        ("codex", "project"),
    ] {
        let fixture = Fixture::new()?;
        let prompts = prompt(
            "deploy",
            "claude",
            "project",
            "file",
            ".claude/commands/deploy.md",
        );
        let bindings = format!("      - {{ agent: {agent}, scope: {scope} }}\n");
        let commands = command("deploy", "deploy", &bindings);
        fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands))?;
        let (code, stdout, stderr) = run(
            &fixture,
            &["doctor", "commands", "--agent", agent, "--scope", scope],
        )?;
        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("capability · unsupported"));
        assert!(!stdout.contains("source"));
        assert!(stderr.is_empty());
    };
    Ok(())
}
#[test]
fn filters_exclude_bindings_before_io() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
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
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands))?;
    fixture.write_repository("harness/prompts/selected.md", CONTENTS)?;
    fixture.write_repository(".claude/commands/selected.md", CONTENTS)?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "project", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("selected · current"));
    assert!(!stdout.contains("missing"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn an_empty_filtered_selection_is_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "cursor", "--scope", "project",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("capability · unsupported"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn diagnostics_preserve_command_then_binding_order()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
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
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands))?;
    for name in ["zeta", "alpha"] {
        fixture.write_repository(format!("harness/prompts/{name}.md"), CONTENTS)?;
        fixture.write_home(format!(".claude/commands/{name}.md"), CONTENTS)?;
    }
    let (_, human, _) = run(&fixture, &["doctor", "commands", "--scope", "user", "-v"])?;
    let positions = [
        "zeta · current",
        "capability · unsupported",
        "alpha · current",
    ]
    .map(
        |needle| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            Ok(human.find(needle).ok_or("required test value is missing")?)
        },
    )
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    assert!(
        *(positions).first().ok_or("missing fixture index 0")?
            < *(positions).get(1).ok_or("missing fixture index 1")?
            && *(positions).get(1).ok_or("missing fixture index 1")?
                < *(positions).get(2).ok_or("missing fixture index 2")?
    );
    let (_, json, _) = run(
        &fixture,
        &["doctor", "commands", "--scope", "user", "--format", "json"],
    )?;
    let diagnostics: Vec<serde_json::Value> = serde_json::from_str(&json)?;
    assert!(
        (*(*(diagnostics).first().ok_or("missing fixture index 0")?)
            .get("message")
            .ok_or("missing fixture index message")?)
        .as_str()
        .ok_or("expected JSON string")?
        .contains("zeta")
    );
    assert_eq!(
        *(*(diagnostics).get(1).ok_or("missing fixture index 1")?)
            .get("state")
            .ok_or("missing fixture index state")?,
        "unsupported"
    );
    assert!(
        (*(*(diagnostics).get(2).ok_or("missing fixture index 2")?)
            .get("message")
            .ok_or("missing fixture index message")?)
        .as_str()
        .ok_or("expected JSON string")?
        .contains("alpha")
    );
    assert_eq!(
        *(*(diagnostics).get(3).ok_or("missing fixture index 3")?)
            .get("state")
            .ok_or("missing fixture index state")?,
        "unsupported"
    );
    Ok(())
}
#[test]
fn unmanaged_and_plugin_neighbors_are_ignored()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".claude/commands/unmanaged.md", "ignored\n")?;
    fixture.write_home(".claude/commands/opsx/plugin.md", "ignored\n")?;
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "commands", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("unmanaged"));
    assert!(!stdout.contains("opsx"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn crlf_frontmatter_is_supported() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let contents = CONTENTS.replace('\n', "\r\n");
    fixture.write_repository("harness/prompts/deploy.md", &contents)?;
    fixture.write_repository(".claude/commands/deploy.md", &contents)?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "project", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy     deploy · current"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn default_doctor_reuses_filtered_command_diagnostics()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (_, direct, _) = run(
        &fixture,
        &[
            "doctor", "commands", "--agent", "claude", "--scope", "user", "--format", "json",
        ],
    )?;
    let (_, aggregate, _) = run(
        &fixture,
        &[
            "doctor", "--agent", "claude", "--scope", "user", "--format", "json",
        ],
    )?;
    let direct: Vec<serde_json::Value> = serde_json::from_str(&direct)?;
    let aggregate: Vec<serde_json::Value> = serde_json::from_str(&aggregate)?;
    let aggregate = aggregate
        .into_iter()
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(diagnostic)
                        .get("resource")
                        .ok_or("missing fixture index resource")?
                        == "commands"
                };
                Ok((include, diagnostic))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();
    assert_eq!(aggregate, direct);
    Ok(())
}
