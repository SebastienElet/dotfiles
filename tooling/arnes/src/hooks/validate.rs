use super::HooksError;
use crate::manifest::Agent;
use serde_json::Value;

mod claude;
mod codex;
mod cursor;
mod fields;

pub fn configuration(config: &Value, agent: Agent) -> Result<(), HooksError> {
    let config = config
        .as_object()
        .ok_or_else(|| HooksError::new("hook configuration must be a JSON object"))?;
    if agent == Agent::Cursor {
        cursor::version(config.get("version"))?;
    }
    let Some(hooks) = config.get("hooks") else {
        return Ok(());
    };
    let hooks = hooks
        .as_object()
        .ok_or_else(|| HooksError::new("hooks must be a JSON object"))?;
    for (event, entries) in hooks {
        match agent {
            Agent::Codex if codex::known_event(event) => codex::event(event, entries)?,
            Agent::Claude if claude::known_event(event) => claude::event(event, entries)?,
            Agent::Cursor if cursor::known_event(event) => cursor::event(event, entries)?,
            _ => {}
        }
    }
    Ok(())
}
