use super::super::ownership;
use serde_json::Value;

pub fn events(config: &Value, nested: bool, command: &str) -> Vec<String> {
    let Some(hooks) = config.get("hooks").and_then(Value::as_object) else {
        return Vec::new();
    };
    hooks
        .iter()
        .filter(|(_, entries)| contains(entries, nested, command))
        .map(|(event, _)| event.clone())
        .collect()
}

fn contains(entries: &Value, nested: bool, command: &str) -> bool {
    let Some(entries) = entries.as_array() else {
        return false;
    };
    if nested {
        return entries
            .iter()
            .filter_map(|group| group.get("hooks").and_then(Value::as_array))
            .flatten()
            .any(|handler| ownership::nested(handler, command));
    }
    entries
        .iter()
        .any(|handler| ownership::direct(handler, command))
}
