use super::{HookAgent, MeasureError};
use clap::Args;
use serde_json::{Map, Value, json};
use std::env;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

mod events;
mod io;
mod json_value;
mod ownership;
mod validate;

#[derive(Args)]
pub struct InstallHooksArgs {
    #[arg(long, value_enum)]
    pub agent: HookAgent,
    #[arg(long)]
    pub command: PathBuf,
    #[arg(long, requires = "claude_legacy_stop_command")]
    pub claude_stop_command: Option<PathBuf>,
    #[arg(long, requires = "claude_stop_command")]
    pub claude_legacy_stop_command: Option<PathBuf>,
}

pub fn install_hooks(args: InstallHooksArgs) -> Result<(), MeasureError> {
    validate_command(&args.command)?;
    let claude_stop_commands = validate_claude_stop_commands(&args)?;
    let home =
        PathBuf::from(env::var_os("HOME").ok_or_else(|| MeasureError::new("HOME is required"))?);
    if !home.is_absolute() {
        return Err(MeasureError::new("HOME must be an absolute path"));
    }
    let policy = events::policy(args.agent);
    let file = io::ConfigFile::open(&home, policy.directory, policy.filename)?;
    let mut config = file
        .content()
        .map(json_value::parse)
        .transpose()?
        .unwrap_or_else(|| json!({}));
    validate::configuration(&config, args.agent)?;
    let command = hook_command(&args.command, args.agent)?;
    ownership::remove_everywhere(&mut config, args.agent, &command)?;
    merge(&mut config, policy.events, policy.nested, &command)?;
    remove_excluded(&mut config, policy.excluded, &command)?;
    if let Some((current, legacy)) = claude_stop_commands {
        merge_claude_stop(&mut config, current, legacy)?;
    }
    file.replace(&serde_json::to_vec_pretty(&config)?)
}

fn validate_claude_stop_commands(
    args: &InstallHooksArgs,
) -> Result<Option<(&str, &str)>, MeasureError> {
    let (Some(current), Some(legacy)) = (
        args.claude_stop_command.as_deref(),
        args.claude_legacy_stop_command.as_deref(),
    ) else {
        return Ok(None);
    };
    if args.agent != HookAgent::ClaudeCode {
        return Err(MeasureError::new(
            "Claude Stop commands require the claude-code agent",
        ));
    }
    validate_command(current)?;
    if !legacy.is_absolute() || current == legacy {
        return Err(MeasureError::new(
            "legacy Claude Stop command must be a distinct absolute path",
        ));
    }
    let current = current
        .to_str()
        .ok_or_else(|| MeasureError::new("Claude Stop command path must be valid UTF-8"))?;
    let legacy = legacy
        .to_str()
        .ok_or_else(|| MeasureError::new("legacy Claude Stop command path must be valid UTF-8"))?;
    Ok(Some((current, legacy)))
}

fn validate_command(command: &Path) -> Result<(), MeasureError> {
    let metadata = command
        .metadata()
        .map_err(|error| MeasureError::new(format!("hook command is unavailable: {error}")))?;
    if !command.is_absolute() || !metadata.is_file() || metadata.permissions().mode() & 0o111 == 0 {
        return Err(MeasureError::new(
            "hook command must be an existing executable absolute file",
        ));
    }
    Ok(())
}

fn hook_command(command: &Path, agent: HookAgent) -> Result<String, MeasureError> {
    let command = command
        .to_str()
        .ok_or_else(|| MeasureError::new("hook command path must be valid UTF-8"))?;
    let quoted = format!("'{}'", command.replace('\'', "'\\''"));
    Ok(format!("{quoted} measure hook --agent {}", agent.as_str()))
}

fn merge(
    config: &mut Value,
    events: &[&str],
    nested: bool,
    command: &str,
) -> Result<(), MeasureError> {
    let config = config
        .as_object_mut()
        .ok_or_else(|| MeasureError::new("hook configuration must be a JSON object"))?;
    if !config.contains_key("hooks") {
        config.insert("hooks".to_owned(), Value::Object(Map::new()));
    }
    if !nested && !config.contains_key("version") {
        config.insert("version".to_owned(), json!(1));
    }
    if !nested && !config["version"].is_number() {
        return Err(MeasureError::new("Cursor hook version must be a number"));
    }
    let hooks = config["hooks"]
        .as_object_mut()
        .ok_or_else(|| MeasureError::new("hooks must be a JSON object"))?;
    for event in events {
        let entries = hooks
            .entry((*event).to_owned())
            .or_insert_with(|| Value::Array(Vec::new()));
        if nested {
            merge_nested(entries, command)?;
        } else {
            merge_direct(entries, command)?;
        }
    }
    Ok(())
}

fn merge_nested(entries: &mut Value, command: &str) -> Result<(), MeasureError> {
    let entries = entries
        .as_array_mut()
        .ok_or_else(|| MeasureError::new("nested hook event must be an array"))?;
    let mut retained = Vec::with_capacity(entries.len() + 1);
    for mut group in std::mem::take(entries) {
        if !remove_measurement_from_group(&mut group, command)? {
            retained.push(group);
        }
    }
    retained.push(json!({"hooks":[{"type":"command","command":command}]}));
    *entries = retained;
    Ok(())
}

fn remove_measurement_from_group(group: &mut Value, command: &str) -> Result<bool, MeasureError> {
    let group = group
        .as_object_mut()
        .ok_or_else(|| MeasureError::new("nested hook group must be an object"))?;
    if group.get("matcher").is_some_and(|value| !value.is_string()) {
        return Err(MeasureError::new(
            "nested hook group matcher must be a string",
        ));
    }
    let handlers = group
        .get_mut("hooks")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| MeasureError::new("nested hook group must contain a hooks array"))?;
    let previous = handlers.len();
    handlers.retain(|handler| !ownership::nested(handler, command));
    Ok(previous != handlers.len() && handlers.is_empty())
}

fn merge_direct(entries: &mut Value, command: &str) -> Result<(), MeasureError> {
    let entries = entries
        .as_array_mut()
        .ok_or_else(|| MeasureError::new("direct hook event must be an array"))?;
    entries.retain(|handler| !ownership::direct(handler, command));
    entries.push(json!({"command":command}));
    Ok(())
}

fn merge_claude_stop(config: &mut Value, current: &str, legacy: &str) -> Result<(), MeasureError> {
    let entries = config["hooks"]["Stop"]
        .as_array_mut()
        .ok_or_else(|| MeasureError::new("Claude Stop hook event must be an array"))?;
    let current_handler = find_nested_handler(entries, current);
    let legacy_handler = find_nested_handler(entries, legacy);
    for group in entries.iter_mut() {
        let handlers = group["hooks"].as_array_mut().ok_or_else(|| {
            MeasureError::new("Claude Stop hook group must contain a hooks array")
        })?;
        handlers.retain(|handler| {
            !ownership::nested(handler, current) && !ownership::nested(handler, legacy)
        });
    }
    entries.retain(|group| !group["hooks"].as_array().is_some_and(Vec::is_empty));
    let mut handler = current_handler
        .or(legacy_handler)
        .unwrap_or_else(|| json!({"type":"command"}));
    let handler = handler
        .as_object_mut()
        .ok_or_else(|| MeasureError::new("Claude Stop hook handler must be an object"))?;
    for field in ["async", "asyncRewake", "once", "if"] {
        handler.remove(field);
    }
    handler.insert("command".into(), json!(current));
    handler.insert("args".into(), json!([]));
    entries.push(json!({"hooks":[handler]}));
    Ok(())
}

fn find_nested_handler(entries: &[Value], command: &str) -> Option<Value> {
    entries
        .iter()
        .filter_map(|group| group.get("hooks").and_then(Value::as_array))
        .flatten()
        .find(|handler| ownership::nested(handler, command))
        .cloned()
}

fn remove_excluded(config: &mut Value, events: &[&str], command: &str) -> Result<(), MeasureError> {
    let hooks = config["hooks"]
        .as_object_mut()
        .ok_or_else(|| MeasureError::new("hooks must be a JSON object"))?;
    for event in events {
        let Some(entries) = hooks.get_mut(*event) else {
            continue;
        };
        let entries = entries
            .as_array_mut()
            .ok_or_else(|| MeasureError::new("excluded hook event must be an array"))?;
        entries.retain(|handler| !ownership::direct(handler, command));
    }
    Ok(())
}
