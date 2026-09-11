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

pub fn output_discipline_matches(config: &Value, command: &str) -> bool {
    let Some(groups) = config
        .get("hooks")
        .and_then(|hooks| hooks.get("SessionStart"))
        .and_then(Value::as_array)
    else {
        return false;
    };
    let installed = groups
        .iter()
        .flat_map(|group| {
            group
                .get("hooks")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(move |handler| ownership::nested(handler, command))
                .map(move |handler| (group, handler))
        })
        .collect::<Vec<_>>();
    matches!(installed.as_slice(), [(group, handler)] if group.get("matcher").and_then(Value::as_str) == Some(super::super::OUTPUT_DISCIPLINE_MATCHER)
        && handler.get("timeout").and_then(Value::as_u64) == Some(30)
        && ["async", "asyncRewake", "once", "if"].iter().all(|field| handler.get(*field).is_none()))
}
