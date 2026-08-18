use super::super::super::MeasureError;
use super::fields;
use serde_json::{Map, Value};

const EVENTS: &[&str] = &[
    "sessionStart",
    "sessionEnd",
    "preToolUse",
    "postToolUse",
    "postToolUseFailure",
    "subagentStart",
    "subagentStop",
    "beforeShellExecution",
    "afterShellExecution",
    "beforeMCPExecution",
    "afterMCPExecution",
    "beforeReadFile",
    "afterFileEdit",
    "beforeSubmitPrompt",
    "preCompact",
    "postCompact",
    "stop",
    "afterAgentResponse",
    "afterAgentThought",
    "beforeTabFileRead",
    "afterTabFileEdit",
    "workspaceOpen",
];

pub fn version(version: Option<&Value>) -> Result<(), MeasureError> {
    if version.is_some_and(|value| !value.is_number()) {
        return Err(MeasureError::new("Cursor hook version must be a number"));
    }
    Ok(())
}

pub fn known_event(event: &str) -> bool {
    EVENTS.contains(&event)
}

pub fn event(event: &str, entries: &Value) -> Result<(), MeasureError> {
    let entries = entries
        .as_array()
        .ok_or_else(|| MeasureError::new("Cursor hook event must be an array"))?;
    for handler in entries {
        handler_fields(event, handler)?;
    }
    Ok(())
}

fn handler_fields(event: &str, handler: &Value) -> Result<(), MeasureError> {
    let handler = handler
        .as_object()
        .ok_or_else(|| MeasureError::new("Cursor hook handler must be an object"))?;
    let kind = match handler.get("type") {
        None => "command",
        Some(Value::String(kind)) => kind,
        Some(_) => {
            return Err(MeasureError::new(
                "Cursor hook handler type must be a string",
            ));
        }
    };
    if !matches!(kind, "command" | "prompt") {
        return Ok(());
    }
    common(event, handler)?;
    match kind {
        "command" => command(handler),
        "prompt" => prompt(handler),
        _ => unreachable!(),
    }
}

fn common(event: &str, handler: &Map<String, Value>) -> Result<(), MeasureError> {
    fields::optional_number(handler, "timeout")?;
    fields::optional_bool(handler, "failClosed")?;
    // Cursor documents matcher as an object while its examples use strings.
    if handler
        .get("matcher")
        .is_some_and(|value| !value.is_string() && !value.is_object())
    {
        return Err(fields::invalid("matcher", "a string or object"));
    }
    if let Some(limit) = handler.get("loop_limit") {
        if event != "stop" && event != "subagentStop" {
            return Err(MeasureError::new(
                "Cursor hook handler loop_limit is incompatible with its event",
            ));
        }
        if !limit.is_number() && !limit.is_null() {
            return Err(fields::invalid("loop_limit", "a number or null"));
        }
    }
    Ok(())
}

fn command(handler: &Map<String, Value>) -> Result<(), MeasureError> {
    fields::required_string(handler, "command")?;
    fields::reject(handler, &["prompt", "model"])
}

fn prompt(handler: &Map<String, Value>) -> Result<(), MeasureError> {
    fields::required_string(handler, "prompt")?;
    fields::optional_string(handler, "model")?;
    fields::reject(handler, &["command"])
}
