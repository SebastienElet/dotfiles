use super::{HooksError, ownership, path_string};
use crate::manifest::Agent;
use serde_json::Value;
use std::path::Path;

pub(super) fn remove(config: &mut Value, agent: Agent, home: &Path) -> Result<(), HooksError> {
    let host = match agent {
        Agent::Claude => "claude-code",
        Agent::Codex => "codex-cli",
        Agent::Cursor => return Ok(()),
    };
    let mut commands = Vec::new();
    for binary in ["remem", "remem-hook"] {
        let path = path_string(&home.join(".local/bin").join(binary))?;
        let path = if path
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"/._-".contains(&byte))
        {
            path
        } else {
            format!("'{}'", path.replace('\'', "'\\''"))
        };
        for operation in ["context", "session-init", "observe", "summarize"] {
            commands.push(format!("{path} {operation} --host {host}"));
        }
        if binary == "remem" {
            commands.push(format!("{path} rules eval --host {host}"));
        }
    }
    let Some(hooks) = config.get_mut("hooks").and_then(Value::as_object_mut) else {
        return Ok(());
    };
    for entries in hooks.values_mut().filter_map(Value::as_array_mut) {
        entries.retain_mut(|group| {
            let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
                return true;
            };
            let previous = handlers.len();
            handlers.retain(|handler| {
                !commands.iter().any(|command| {
                    ownership::nested(handler, command)
                        && handler
                            .get("args")
                            .is_none_or(|args| args.as_array().is_some_and(Vec::is_empty))
                })
            });
            previous == handlers.len() || !handlers.is_empty()
        });
    }
    Ok(())
}
