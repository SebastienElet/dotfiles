use super::{HooksError, ownership};
use serde_json::{Map, Value, json};

pub(super) fn measurement(
    config: &mut Value,
    events: &[&str],
    nested: bool,
    excluded: &[&str],
    command: &str,
) -> Result<(), HooksError> {
    let config = config
        .as_object_mut()
        .ok_or_else(|| HooksError::new("hook configuration must be a JSON object"))?;
    if !config.contains_key("hooks") {
        config.insert("hooks".to_owned(), Value::Object(Map::new()));
    }
    if !nested && !config.contains_key("version") {
        config.insert("version".to_owned(), json!(1));
    }
    if !nested && !config["version"].is_number() {
        return Err(HooksError::new("Cursor hook version must be a number"));
    }
    let hooks = config["hooks"]
        .as_object_mut()
        .ok_or_else(|| HooksError::new("hooks must be a JSON object"))?;
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
    remove_excluded(hooks, excluded, command)
}

pub(super) fn memory(
    config: &mut Value,
    event: &str,
    command: &str,
    timeout_seconds: u64,
) -> Result<(), HooksError> {
    let config = config
        .as_object_mut()
        .ok_or_else(|| HooksError::new("hook configuration must be a JSON object"))?;
    let hooks = config
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| HooksError::new("hooks must be a JSON object"))?;
    let entries = hooks.entry(event.to_owned()).or_insert_with(|| json!([]));
    merge_memory(entries, command, timeout_seconds)
}

fn merge_memory(
    entries: &mut Value,
    command: &str,
    timeout_seconds: u64,
) -> Result<(), HooksError> {
    let entries = entries
        .as_array_mut()
        .ok_or_else(|| HooksError::new("memory hook event must be an array"))?;
    let mut retained = Vec::with_capacity(entries.len() + 1);
    for mut group in std::mem::take(entries) {
        if !remove_measurement_from_group(&mut group, command)? {
            retained.push(group);
        }
    }
    retained
        .push(json!({"hooks":[{"type":"command","command":command,"timeout":timeout_seconds}]}));
    *entries = retained;
    Ok(())
}

fn merge_nested(entries: &mut Value, command: &str) -> Result<(), HooksError> {
    let entries = entries
        .as_array_mut()
        .ok_or_else(|| HooksError::new("nested hook event must be an array"))?;
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

fn remove_measurement_from_group(group: &mut Value, command: &str) -> Result<bool, HooksError> {
    let group = group
        .as_object_mut()
        .ok_or_else(|| HooksError::new("nested hook group must be an object"))?;
    if group.get("matcher").is_some_and(|value| !value.is_string()) {
        return Err(HooksError::new(
            "nested hook group matcher must be a string",
        ));
    }
    let handlers = group
        .get_mut("hooks")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| HooksError::new("nested hook group must contain a hooks array"))?;
    let previous = handlers.len();
    handlers.retain(|handler| !ownership::nested(handler, command));
    Ok(previous != handlers.len() && handlers.is_empty())
}

fn merge_direct(entries: &mut Value, command: &str) -> Result<(), HooksError> {
    let entries = entries
        .as_array_mut()
        .ok_or_else(|| HooksError::new("direct hook event must be an array"))?;
    entries.retain(|handler| !ownership::direct(handler, command));
    entries.push(json!({"command":command}));
    Ok(())
}

pub(super) fn handoff(
    config: &mut Value,
    commands: &[String],
    include_args: bool,
    execution_fields: &[&str],
) -> Result<(), HooksError> {
    let current = commands
        .first()
        .ok_or_else(|| HooksError::new("handoff hook command is required"))?;
    let config = config
        .as_object_mut()
        .ok_or_else(|| HooksError::new("hook configuration must be an object"))?;
    let hooks = config
        .entry("hooks")
        .or_insert_with(|| json!({}))
        .as_object_mut()
        .ok_or_else(|| HooksError::new("hooks must be a JSON object"))?;
    let entries = hooks
        .entry("Stop")
        .or_insert_with(|| json!([]))
        .as_array_mut()
        .ok_or_else(|| HooksError::new("Stop hook event must be an array"))?;
    let existing = commands
        .iter()
        .find_map(|command| find_nested_handler(entries, command));
    for group in entries.iter_mut() {
        let handlers = group["hooks"]
            .as_array_mut()
            .ok_or_else(|| HooksError::new("Stop hook group must contain a hooks array"))?;
        handlers.retain(|handler| {
            !commands
                .iter()
                .any(|command| ownership::nested(handler, command))
        });
    }
    entries.retain(|group| !group["hooks"].as_array().is_some_and(Vec::is_empty));
    let mut handler = existing.unwrap_or_else(|| json!({"type":"command"}));
    let handler = handler
        .as_object_mut()
        .ok_or_else(|| HooksError::new("Stop hook handler must be an object"))?;
    for &field in execution_fields {
        handler.remove(field);
    }
    handler.insert("command".into(), json!(current));
    if include_args {
        handler.insert("args".into(), json!([]));
    } else {
        handler.remove("args");
    }
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

fn remove_excluded(
    hooks: &mut Map<String, Value>,
    events: &[&str],
    command: &str,
) -> Result<(), HooksError> {
    for event in events {
        let Some(entries) = hooks.get_mut(*event) else {
            continue;
        };
        let entries = entries
            .as_array_mut()
            .ok_or_else(|| HooksError::new("excluded hook event must be an array"))?;
        entries.retain(|handler| !ownership::direct(handler, command));
    }
    Ok(())
}
