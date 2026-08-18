use super::super::{HookAgent, MeasureError};
use serde_json::Value;

pub fn remove_everywhere(
    config: &mut Value,
    agent: HookAgent,
    command: &str,
) -> Result<(), MeasureError> {
    let Some(hooks) = config.get_mut("hooks") else {
        return Ok(());
    };
    let hooks = hooks
        .as_object_mut()
        .ok_or_else(|| MeasureError::new("hooks must be a JSON object"))?;
    for entries in hooks.values_mut() {
        match agent {
            HookAgent::Codex | HookAgent::ClaudeCode => remove_nested(entries, command),
            HookAgent::Cursor => remove_direct(entries, command),
        }
    }
    Ok(())
}

pub fn nested(handler: &Value, command: &str) -> bool {
    handler.get("type").and_then(Value::as_str) == Some("command")
        && handler.get("command").and_then(Value::as_str) == Some(command)
}

pub fn direct(handler: &Value, command: &str) -> bool {
    let kind = handler.get("type").and_then(Value::as_str);
    matches!(kind, None | Some("command"))
        && handler.get("command").and_then(Value::as_str) == Some(command)
}

fn remove_nested(entries: &mut Value, command: &str) {
    let Some(entries) = entries.as_array_mut() else {
        return;
    };
    entries.retain_mut(|group| {
        let Some(handlers) = group.get_mut("hooks").and_then(Value::as_array_mut) else {
            return true;
        };
        let previous = handlers.len();
        handlers.retain(|handler| !nested(handler, command));
        previous == handlers.len() || !handlers.is_empty()
    });
}

fn remove_direct(entries: &mut Value, command: &str) {
    let Some(entries) = entries.as_array_mut() else {
        return;
    };
    entries.retain(|handler| !direct(handler, command));
}
