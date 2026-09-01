use crate::HandoffError;
use serde_json::{Map, Value};
use std::path::PathBuf;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HookEvent {
    pub session_id: String,
    pub stop_hook_active: bool,
    pub transcript_path: PathBuf,
}

pub fn parse_hook_event(input: &[u8]) -> Result<HookEvent, HandoffError> {
    let input = String::from_utf8_lossy(input);
    let value: Value = serde_json::from_str(&input)
        .map_err(|_| HandoffError::usage("invalid hook event: expected JSON"))?;
    let object = value
        .as_object()
        .ok_or_else(|| HandoffError::usage("invalid hook event: expected an object"))?;

    validate_stop_event(object)?;

    let session_id = object
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| HandoffError::usage("missing session_id"))?;
    if !is_valid_session_id(session_id) {
        return Err(HandoffError::usage("invalid session_id"));
    }

    let transcript_path = object
        .get("transcript_path")
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| HandoffError::usage("missing transcript_path"))?;
    let stop_hook_active = match object.get("stop_hook_active") {
        None => false,
        Some(Value::Bool(value)) => *value,
        Some(_) => return Err(HandoffError::usage("invalid stop_hook_active")),
    };

    Ok(HookEvent {
        session_id: session_id.into(),
        stop_hook_active,
        transcript_path: transcript_path.into(),
    })
}

fn validate_stop_event(object: &Map<String, Value>) -> Result<(), HandoffError> {
    let claude_event = object.get("hook_event_name");
    let codex_event = object.get("event");
    if claude_event.is_none() && codex_event.is_none() {
        return Err(HandoffError::usage("missing Stop event"));
    }
    if claude_event.is_some_and(|value| value.as_str() != Some("Stop"))
        || codex_event.is_some_and(|value| value.as_str() != Some("Stop"))
    {
        return Err(HandoffError::usage("unsupported hook event"));
    }
    Ok(())
}

fn is_valid_session_id(value: &str) -> bool {
    value != "."
        && value != ".."
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}
