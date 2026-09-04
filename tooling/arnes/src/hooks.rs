use crate::Roots;
use crate::manifest::{self, Agent, HookKind, Scope};
use clap::Args;
use serde_json::json;
use std::fmt::{self, Display};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

mod adapters;
mod inspect;
mod io;
mod json_value;
mod ownership;
mod reconcile;
mod validate;

const MEMORY_HOOK_TIMEOUT_SECONDS: u64 = 30;
pub use inspect::diagnose;

#[derive(Args)]
pub struct SetupHooksArgs {
    #[arg(long, value_enum)]
    pub agent: Agent,
    #[arg(long, value_enum, default_value = "user")]
    pub scope: Scope,
}

pub fn setup(args: SetupHooksArgs) -> Result<(), HooksError> {
    if args.scope != Scope::User {
        return Err(HooksError::new("hooks only support the user scope"));
    }
    let roots = Roots::from_environment().map_err(HooksError::from_display)?;
    let manifest = manifest::load(roots.home()).map_err(HooksError::from_display)?;
    if !manifest
        .combinations()
        .any(|combination| combination == (args.agent, args.scope))
    {
        return Err(HooksError::new(format!(
            "{} agent is not declared for {} scope",
            args.agent, args.scope
        )));
    }
    let desired: Vec<HookKind> = manifest.hooks(args.agent, args.scope).collect();
    let policy = adapters::policy(args.agent);
    let measurement_path = measurement_path(roots.home());
    let handoff_path = handoff_path(roots.home());
    let memory_path = memory_path(roots.home());
    let measurement = measurement_command(&measurement_path, args.agent)?;
    let handoff_aliases = handoff_aliases(&handoff_path, roots.repository(), args.agent)?;
    let memory = memory_command(&memory_path, args.agent)?;
    let memory_event = policy.memory_event;
    if desired.contains(&HookKind::Memory) && (memory.is_none() || memory_event.is_none()) {
        return Err(HooksError::new("Cursor does not support the memory hook"));
    }
    let file = io::ConfigFile::open(roots.home(), policy.directory, policy.filename)?;
    let mut config = file
        .content()
        .map(json_value::parse)
        .transpose()?
        .unwrap_or_else(|| json!({}));
    validate::configuration(&config, args.agent)?;
    ownership::remove_everywhere(&mut config, args.agent, &measurement)?;
    if let Some(command) = &memory {
        ownership::remove_everywhere(&mut config, args.agent, command)?;
    }
    if desired.contains(&HookKind::Measurement) {
        validate_command(&measurement_path)?;
        reconcile::measurement(
            &mut config,
            policy.events,
            policy.nested,
            policy.excluded,
            &measurement,
        )?;
    }
    if desired.contains(&HookKind::Memory) {
        let Some(memory) = memory else {
            return Err(HooksError::new("Cursor does not support the memory hook"));
        };
        let Some(memory_event) = memory_event else {
            return Err(HooksError::new("Cursor does not support the memory hook"));
        };
        validate_command(&memory_path)?;
        reconcile::memory(
            &mut config,
            memory_event,
            &memory,
            MEMORY_HOOK_TIMEOUT_SECONDS,
        )?;
    }
    if desired.contains(&HookKind::Handoff) {
        validate_command(&handoff_path)?;
        reconcile::handoff(
            &mut config,
            &handoff_aliases,
            policy.handoff_args,
            policy.handoff_execution_fields,
        )?;
    } else {
        for command in &handoff_aliases {
            ownership::remove_everywhere(&mut config, args.agent, command)?;
        }
    }
    file.replace(&serde_json::to_vec_pretty(&config)?)
}

fn measurement_path(home: &Path) -> PathBuf {
    home.join(".local/bin/arnes")
}

fn handoff_path(home: &Path) -> PathBuf {
    home.join(".local/bin/agent-handoff")
}

fn memory_path(home: &Path) -> PathBuf {
    home.join(".local/bin/agent-memory")
}

fn handoff_aliases(
    command: &Path,
    repository: &Path,
    agent: Agent,
) -> Result<Vec<String>, HooksError> {
    let mut aliases = vec![path_string(command)?];
    aliases
        .extend((agent == Agent::Claude).then_some("$HOME/.claude/hooks/agent_handoff".to_owned()));
    aliases.extend([
        path_string(&repository.join("tooling/agent-handoff"))?,
        path_string(&repository.join("scripts/agent_handoff"))?,
    ]);
    let metadata = match fs::symlink_metadata(command) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(aliases),
        Err(error) => return Err(error.into()),
    };
    if !metadata.file_type().is_symlink() {
        return Ok(aliases);
    }
    let target = fs::read_link(command)?;
    let target = if target.is_absolute() {
        target
    } else {
        command
            .parent()
            .unwrap_or_else(|| Path::new("/"))
            .join(target)
    };
    aliases.push(path_string(&target)?);
    aliases.dedup();
    Ok(aliases)
}

fn validate_command(command: &Path) -> Result<(), HooksError> {
    let metadata = command
        .metadata()
        .map_err(|error| HooksError::new(format!("hook command is unavailable: {error}")))?;
    if !command.is_absolute() || !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(HooksError::new(
            "hook command must be an existing executable absolute file",
        ));
    }
    Ok(())
}

fn measurement_command(command: &Path, agent: Agent) -> Result<String, HooksError> {
    let command = path_string(command)?;
    let quoted = format!("'{}'", command.replace('\'', "'\\''"));
    let agent = match agent {
        Agent::Claude => "claude-code",
        Agent::Codex => "codex",
        Agent::Cursor => "cursor",
    };
    Ok(format!("{quoted} measure hook --agent {agent}"))
}

fn memory_command(command: &Path, agent: Agent) -> Result<Option<String>, HooksError> {
    let agent = match agent {
        Agent::Codex => "codex",
        Agent::Claude => "claude",
        Agent::Cursor => return Ok(None),
    };
    let command = path_string(command)?;
    let quoted = format!("'{}'", command.replace('\'', "'\\''"));
    Ok(Some(format!("{quoted} hook --agent {agent}")))
}

fn path_string(path: &Path) -> Result<String, HooksError> {
    path.to_str()
        .map(str::to_owned)
        .ok_or_else(|| HooksError::new("hook command path must be valid UTF-8"))
}

#[derive(Debug)]
pub struct HooksError(String);

impl HooksError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }

    fn from_display(error: impl Display) -> Self {
        Self(error.to_string())
    }
}

impl Display for HooksError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for HooksError {}

impl From<std::io::Error> for HooksError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<rustix::io::Errno> for HooksError {
    fn from(error: rustix::io::Errno) -> Self {
        std::io::Error::from(error).into()
    }
}

impl From<serde_json::Error> for HooksError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::handoff_aliases;
    use std::path::Path;

    #[test]
    fn handoff_aliases_include_repository_runtimes() {
        let aliases = handoff_aliases(
            Path::new("/tmp/home/.local/bin/agent-handoff"),
            Path::new("/tmp/repository"),
            crate::manifest::Agent::Claude,
        )
        .unwrap();

        assert!(aliases.contains(&"/tmp/repository/tooling/agent-handoff".to_owned()));
        assert!(aliases.contains(&"/tmp/repository/scripts/agent_handoff".to_owned()));
    }
}
