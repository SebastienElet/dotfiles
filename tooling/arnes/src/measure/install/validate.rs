use super::super::{HookAgent, MeasureError};
use serde_json::Value;

mod claude;
mod codex;
mod cursor;
mod fields;

pub fn configuration(config: &Value, agent: HookAgent) -> Result<(), MeasureError> {
    let config = config
        .as_object()
        .ok_or_else(|| MeasureError::new("hook configuration must be a JSON object"))?;
    if agent == HookAgent::Cursor {
        cursor::version(config.get("version"))?;
    }
    let Some(hooks) = config.get("hooks") else {
        return Ok(());
    };
    let hooks = hooks
        .as_object()
        .ok_or_else(|| MeasureError::new("hooks must be a JSON object"))?;
    for (event, entries) in hooks {
        match agent {
            HookAgent::Codex if codex::known_event(event) => codex::event(event, entries)?,
            HookAgent::ClaudeCode if claude::known_event(event) => claude::event(event, entries)?,
            HookAgent::Cursor if cursor::known_event(event) => cursor::event(event, entries)?,
            _ => {}
        }
    }
    Ok(())
}
