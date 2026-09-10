use super::contracts::Oracle;
use super::fixture::Fixture;
use super::live::Execution;
use super::sources::LoadedCase;
use std::process::{Command, Stdio};

pub fn execute(fixture: &Fixture, entry: &LoadedCase) -> Result<Execution, String> {
    let commands: &[&[&str]] = match entry.definition.oracle {
        Oracle::StructuralV1 => &[
            &["cat", ".agents/skills/code-search/SKILL.md"],
            &["colgrep-search", "dependencies"],
        ],
        Oracle::LiteralV1 => &[&["rg", "FEATURE_FLAG_DISABLED"]],
        Oracle::KnownPathV1 => &[&["cat", "src/auth/session.ts"]],
    };
    for command in commands {
        let Some((program, arguments)) = command.split_first() else {
            return Err("Empty fixture smoke command".into());
        };
        let result = Command::new(program)
            .args(arguments)
            .env_clear()
            .envs(&fixture.env)
            .current_dir(&fixture.workspace)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map_err(|error| error.to_string())?;
        if !result.success() {
            return Err(format!("Fixture smoke failed: {program}"));
        }
    }
    Ok(Execution {
        error: None,
        tokens: None,
        tool_calls: Some(commands.len() as u64),
        duration_ms: None,
    })
}
