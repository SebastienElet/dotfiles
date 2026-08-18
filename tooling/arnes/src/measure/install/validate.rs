use super::super::{HookAgent, MeasureError};
use serde_json::{Map, Value};

pub fn nested_handler(handler: &Value, agent: HookAgent) -> Result<(), MeasureError> {
    let handler = handler
        .as_object()
        .ok_or_else(|| MeasureError::new("nested hook handler must be an object"))?;
    let kind = handler
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| MeasureError::new("nested hook handler type must be a string"))?;
    if handler
        .get("command")
        .is_some_and(|value| !value.is_string())
    {
        return Err(MeasureError::new(
            "nested hook handler command must be a string",
        ));
    }
    match (agent, kind) {
        (HookAgent::Codex, "command") | (HookAgent::ClaudeCode, "command") => {
            required_string(handler, "command")
        }
        (HookAgent::Codex, "prompt" | "agent") | (HookAgent::ClaudeCode, "prompt" | "agent") => {
            required_string(handler, "prompt")
        }
        (HookAgent::ClaudeCode, "http") => required_string(handler, "url"),
        (HookAgent::ClaudeCode, "mcp_tool") => {
            required_string(handler, "server")?;
            required_string(handler, "tool")
        }
        _ => Err(MeasureError::new(format!(
            "unsupported nested hook handler type: {kind}"
        ))),
    }
}

pub fn direct_handler(handler: &Map<String, Value>) -> Result<(), MeasureError> {
    optional_string(handler, "matcher")?;
    optional_string(handler, "command")?;
    let kind = match handler.get("type") {
        None => "command",
        Some(Value::String(kind)) => kind,
        Some(_) => {
            return Err(MeasureError::new(
                "direct hook handler type must be a string",
            ));
        }
    };
    match kind {
        "command" => required_string(handler, "command"),
        "prompt" => required_string(handler, "prompt"),
        _ => Err(MeasureError::new(format!(
            "unsupported direct hook handler type: {kind}"
        ))),
    }
}

fn optional_string(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    if handler.get(field).is_some_and(|value| !value.is_string()) {
        return Err(MeasureError::new(format!(
            "hook handler {field} must be a string"
        )));
    }
    Ok(())
}

fn required_string(handler: &Map<String, Value>, field: &str) -> Result<(), MeasureError> {
    if !handler.get(field).is_some_and(Value::is_string) {
        return Err(MeasureError::new(format!(
            "hook handler must contain a string {field}"
        )));
    }
    Ok(())
}
