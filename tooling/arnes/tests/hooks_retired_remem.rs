mod support;

use serde_json::{Value, json};
use std::fs;
use std::process::Command;
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user]
  - id: codex
    scopes: [user]
  - id: cursor
    scopes: [user]
resources: []
";

fn configuration_path(agent: &str) -> &str {
    match agent {
        "claude" => ".claude/settings.json",
        "codex" => ".codex/hooks.json",
        _ => ".cursor/hooks.json",
    }
}

fn read_configuration(
    fixture: &Fixture,
    agent: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&fs::read(
        fixture.home().join(configuration_path(agent)),
    )?)?)
}

#[test]
fn setup_retires_remem_hooks_without_removing_other_handlers_or_data() -> TestResult {
    for (agent, host) in [("claude", "claude-code"), ("codex", "codex-cli")] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", MANIFEST)?;
        fixture.write_home(".remem/memory.db", "keep memory")?;
        let handlers: Vec<Value> = ["remem", "remem-hook"].into_iter().flat_map(|binary| {
            ["context", "session-init", "observe", "summarize"].map(|operation| {
                json!({"type":"command", "command":format!("{}/.local/bin/{binary} {operation} --host {host}", fixture.home().display())})
            })
        }).chain(std::iter::once(json!({"type":"command", "command":format!("{}/.local/bin/remem rules eval --host {host}", fixture.home().display()), "args":[]}))).collect();
        let third_party = json!({"type":"command", "command":"third-party", "timeout":5});
        let mixed = [handlers.clone(), vec![third_party.clone()]].concat();
        let config = json!({"theme":"dark", "hooks":{
            "SessionStart":[{"matcher":"startup", "hooks":handlers}],
            "Stop":[{"matcher":"keep", "hooks":mixed}]
        }});
        fixture.write_home(configuration_path(agent), &serde_json::to_string(&config)?)?;
        let output = fixture.command(["setup", "hooks", "--agent", agent])?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert_eq!(
            read_configuration(&fixture, agent)?,
            json!({"theme":"dark", "hooks":{
                "SessionStart":[], "Stop":[{"matcher":"keep", "hooks":[third_party]}]
            }})
        );
        assert_eq!(
            fs::read_to_string(fixture.home().join(".remem/memory.db"))?,
            "keep memory"
        );
        let once = fs::read(fixture.home().join(configuration_path(agent)))?;
        assert_eq!(
            fixture
                .command(["setup", "hooks", "--agent", agent])?
                .status
                .code(),
            Some(0)
        );
        assert_eq!(
            fs::read(fixture.home().join(configuration_path(agent)))?,
            once
        );
    }
    Ok(())
}

#[test]
fn setup_preserves_foreign_remem_variants_and_non_command_handlers() -> TestResult {
    for (agent, host) in [("claude", "claude-code"), ("codex", "codex-cli")] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", MANIFEST)?;
        let command = format!(
            "{}/.local/bin/remem context --host {host}",
            fixture.home().display()
        );
        let foreign = json!({"hooks":{"Stop":[{"matcher":"keep", "hooks":[
            {"type":"command", "command":command, "args":["foreign"]},
            {"type":"command", "command":format!("{command} --custom")},
            {"type":"command", "command":format!("{command}; echo custom")},
            {"type":"command", "command":format!("{}/.local/bin/remem context --host foreign", fixture.home().display())},
            {"type":"command", "command":format!("/other/.local/bin/remem context --host {host}")},
            {"type":"future", "command":command}
        ]}]}});
        fixture.write_home(configuration_path(agent), &serde_json::to_string(&foreign)?)?;
        assert_eq!(
            fixture
                .command(["setup", "hooks", "--agent", agent])?
                .status
                .code(),
            Some(0)
        );
        assert_eq!(read_configuration(&fixture, agent)?, foreign);
    }
    Ok(())
}

#[test]
fn setup_preserves_cursor_remem_commands() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    let config = json!({"version":1, "hooks":{"stop":[{
        "command":format!("{}/.local/bin/remem summarize --host codex-cli", fixture.home().display())
    }]}});
    fixture.write_home(
        configuration_path("cursor"),
        &serde_json::to_string(&config)?,
    )?;
    assert_eq!(
        fixture
            .command(["setup", "hooks", "--agent", "cursor"])?
            .status
            .code(),
        Some(0)
    );
    assert_eq!(read_configuration(&fixture, "cursor")?, config);
    Ok(())
}

#[test]
fn setup_refuses_invalid_configuration_without_retiring_any_hook() -> TestResult {
    for agent in ["claude", "codex"] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", MANIFEST)?;
        assert_eq!(
            fixture
                .command(["setup", "hooks", "--agent", agent])?
                .status
                .code(),
            Some(0)
        );
        for invalid in ["{invalid", "{\"hooks\":{\"Stop\":\"invalid\"}}"] {
            fixture.write_home(configuration_path(agent), invalid)?;
            let before = fixture.snapshot()?;
            let output = fixture.command(["setup", "hooks", "--agent", agent])?;
            assert_eq!(output.status.code(), Some(2));
            assert!(!output.stderr.is_empty());
            assert_eq!(fixture.snapshot()?, before);
        }
    }
    Ok(())
}

#[test]
fn setup_retires_historical_quoted_commands_in_home_with_spaces_and_apostrophes() -> TestResult {
    for (agent, host) in [("claude", "claude-code"), ("codex", "codex-cli")] {
        let fixture = Fixture::new()?;
        let name = "user's home";
        let home = fixture.home().join(name);
        fixture.write_home(format!("{name}/.arnes.yaml"), MANIFEST)?;
        let command = format!(
            "'{}/user'\\''s home/.local/bin/remem-hook' context --host {host}",
            fixture.home().display()
        );
        let config =
            json!({"hooks":{"SessionStart":[{"hooks":[{"type":"command", "command":command}]}]}});
        let path = format!("{name}/{}", configuration_path(agent));
        fixture.write_home(path, &serde_json::to_string(&config)?)?;
        let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["setup", "hooks", "--agent", agent])
            .current_dir(fixture.repository())
            .env_clear()
            .env("HOME", &home)
            .env("PATH", home.join("bin"))
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let config: Value =
            serde_json::from_slice(&fs::read(home.join(configuration_path(agent)))?)?;
        assert_eq!(config, json!({"hooks":{"SessionStart":[]}}));
    }
    Ok(())
}
