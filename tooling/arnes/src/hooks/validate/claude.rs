use super::super::HooksError;
use super::fields;
use serde_json::{Map, Value};

const ALL_TYPES: &[&str] = &[
    "PermissionDenied",
    "PermissionRequest",
    "PostToolBatch",
    "PostToolUse",
    "PostToolUseFailure",
    "PreToolUse",
    "Stop",
    "SubagentStop",
    "TaskCompleted",
    "TaskCreated",
    "TeammateIdle",
    "UserPromptExpansion",
    "UserPromptSubmit",
];
const IO_TYPES: &[&str] = &[
    "ConfigChange",
    "CwdChanged",
    "DirectoryAdded",
    "Elicitation",
    "ElicitationResult",
    "FileChanged",
    "InstructionsLoaded",
    "MessageDisplay",
    "Notification",
    "PostCompact",
    "PreCompact",
    "SessionEnd",
    "StopFailure",
    "SubagentStart",
    "WorktreeCreate",
    "WorktreeRemove",
];
const START_TYPES: &[&str] = &["SessionStart", "Setup"];
const VARIANT_FIELDS: &[&str] = &[
    "command",
    "args",
    "async",
    "asyncRewake",
    "shell",
    "url",
    "headers",
    "allowedEnvVars",
    "server",
    "tool",
    "input",
    "prompt",
    "model",
    "continueOnBlock",
];

pub fn known_event(event: &str) -> bool {
    ALL_TYPES.contains(&event) || IO_TYPES.contains(&event) || START_TYPES.contains(&event)
}

pub fn event(event: &str, entries: &Value) -> Result<(), HooksError> {
    let entries = entries
        .as_array()
        .ok_or_else(|| HooksError::new("Claude hook event must be an array"))?;
    for group in entries {
        group_handlers(event, group)?;
    }
    Ok(())
}

fn group_handlers(event: &str, group: &Value) -> Result<(), HooksError> {
    let group = group
        .as_object()
        .ok_or_else(|| HooksError::new("Claude hook group must be an object"))?;
    fields::optional_string(group, "matcher")?;
    let handlers = group
        .get("hooks")
        .and_then(Value::as_array)
        .ok_or_else(|| HooksError::new("Claude hook group must contain a hooks array"))?;
    for handler in handlers {
        handler_fields(event, handler)?;
    }
    Ok(())
}

fn handler_fields(event: &str, handler: &Value) -> Result<(), HooksError> {
    let handler = handler
        .as_object()
        .ok_or_else(|| HooksError::new("Claude hook handler must be an object"))?;
    let kind = handler
        .get("type")
        .ok_or_else(|| HooksError::new("Claude hook handler type is required"))?
        .as_str()
        .ok_or_else(|| HooksError::new("Claude hook handler type must be a string"))?;
    if !matches!(kind, "command" | "http" | "mcp_tool" | "prompt" | "agent") {
        return Ok(());
    }
    allowed(event, kind)?;
    common(event, handler)?;
    match kind {
        "command" => command(handler),
        "http" => http(handler),
        "mcp_tool" => mcp(handler),
        "prompt" => prompt(handler),
        "agent" => agent(handler),
        _ => unreachable!(),
    }
}

fn allowed(event: &str, kind: &str) -> Result<(), HooksError> {
    let permitted = ALL_TYPES.contains(&event)
        || IO_TYPES.contains(&event) && matches!(kind, "command" | "http" | "mcp_tool")
        || START_TYPES.contains(&event) && matches!(kind, "command" | "mcp_tool");
    if !permitted {
        return Err(HooksError::new(format!(
            "Claude {event} does not support {kind} hook handlers"
        )));
    }
    Ok(())
}

fn common(event: &str, handler: &Map<String, Value>) -> Result<(), HooksError> {
    fields::optional_string(handler, "if")?;
    fields::optional_number(handler, "timeout")?;
    fields::optional_string(handler, "statusMessage")?;
    fields::optional_bool(handler, "once")?;
    if event == "SessionEnd" {
        fields::max(handler, "timeout", 60.0)?;
    }
    Ok(())
}

fn command(handler: &Map<String, Value>) -> Result<(), HooksError> {
    only(
        handler,
        &["command", "args", "async", "asyncRewake", "shell"],
    )?;
    fields::required_string(handler, "command")?;
    fields::optional_strings(handler, "args")?;
    fields::optional_bool(handler, "async")?;
    fields::optional_bool(handler, "asyncRewake")?;
    if handler
        .get("shell")
        .and_then(Value::as_str)
        .is_some_and(|shell| !matches!(shell, "bash" | "powershell"))
    {
        return Err(HooksError::new("Claude hook handler shell is unsupported"));
    }
    fields::optional_string(handler, "shell")
}

fn http(handler: &Map<String, Value>) -> Result<(), HooksError> {
    only(handler, &["url", "headers", "allowedEnvVars"])?;
    fields::required_string(handler, "url")?;
    fields::optional_string_map(handler, "headers")?;
    fields::optional_strings(handler, "allowedEnvVars")
}

fn mcp(handler: &Map<String, Value>) -> Result<(), HooksError> {
    only(handler, &["server", "tool", "input"])?;
    fields::required_string(handler, "server")?;
    fields::required_string(handler, "tool")
}

fn prompt(handler: &Map<String, Value>) -> Result<(), HooksError> {
    only(handler, &["prompt", "model", "continueOnBlock"])?;
    fields::required_string(handler, "prompt")?;
    fields::optional_string(handler, "model")?;
    fields::optional_bool(handler, "continueOnBlock")
}

fn agent(handler: &Map<String, Value>) -> Result<(), HooksError> {
    only(handler, &["prompt", "model"])?;
    fields::required_string(handler, "prompt")?;
    fields::optional_string(handler, "model")
}

fn only(handler: &Map<String, Value>, allowed: &[&str]) -> Result<(), HooksError> {
    let incompatible: Vec<&str> = VARIANT_FIELDS
        .iter()
        .copied()
        .filter(|field| !allowed.contains(field))
        .collect();
    fields::reject(handler, &incompatible)
}
