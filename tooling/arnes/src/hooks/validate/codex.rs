use super::super::HooksError;
use super::fields;
use serde_json::{Map, Value};

const EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PreCompact",
    "PostCompact",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "SessionEnd",
];

pub fn known_event(event: &str) -> bool {
    EVENTS.contains(&event)
}

pub fn event(event: &str, entries: &Value) -> Result<(), HooksError> {
    let entries = entries
        .as_array()
        .ok_or_else(|| HooksError::new("Codex hook event must be an array"))?;
    for group in entries {
        group_handlers(event, group)?;
    }
    Ok(())
}

fn group_handlers(event: &str, group: &Value) -> Result<(), HooksError> {
    let group = group
        .as_object()
        .ok_or_else(|| HooksError::new("Codex hook group must be an object"))?;
    fields::optional_string(group, "matcher")?;
    let handlers = group
        .get("hooks")
        .and_then(Value::as_array)
        .ok_or_else(|| HooksError::new("Codex hook group must contain a hooks array"))?;
    for handler in handlers {
        handler_fields(event, handler)?;
    }
    Ok(())
}

fn handler_fields(event: &str, handler: &Value) -> Result<(), HooksError> {
    let handler = handler
        .as_object()
        .ok_or_else(|| HooksError::new("Codex hook handler must be an object"))?;
    let kind = handler
        .get("type")
        .ok_or_else(|| HooksError::new("Codex hook handler type is required"))?
        .as_str()
        .ok_or_else(|| HooksError::new("Codex hook handler type must be a string"))?;
    match kind {
        "command" => command(event, handler),
        _ => Ok(()),
    }
}

fn common(handler: &Map<String, Value>) -> Result<(), HooksError> {
    fields::optional_nonnegative_integer(handler, "timeout")?;
    fields::optional_string(handler, "statusMessage")
}

fn command(event: &str, handler: &Map<String, Value>) -> Result<(), HooksError> {
    common(handler)?;
    fields::required_string(handler, "command")?;
    fields::optional_string(handler, "commandWindows")?;
    fields::optional_string(handler, "command_windows")?;
    if handler.contains_key("commandWindows") && handler.contains_key("command_windows") {
        return Err(HooksError::new(
            "Codex hook handler contains duplicate Windows command aliases",
        ));
    }
    fields::optional_bool(handler, "async")?;
    fields::optional_nonnegative_integer(handler, "additionalContextLimit")?;
    if event == "SessionEnd" {
        fields::max(handler, "timeout", 3.0)?;
    }
    Ok(())
}
