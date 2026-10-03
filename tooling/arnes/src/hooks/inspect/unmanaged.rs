use super::drift;
use crate::diagnostic::Diagnostic;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

pub fn diagnose(config: &Value, nested: bool, known: &[String], subject: &str) -> Vec<Diagnostic> {
    let Some(hooks) = config.get("hooks").and_then(Value::as_object) else {
        return Vec::new();
    };
    let mut events_by_command = BTreeMap::<&str, BTreeSet<&str>>::new();
    for (event, entries) in hooks {
        for command in
            commands(entries, nested).filter(|command| !known.iter().any(|k| k == command))
        {
            events_by_command.entry(command).or_default().insert(event);
        }
    }
    events_by_command
        .into_iter()
        .map(|(command, events)| {
            drift(format!(
                "{subject} hook command {command} is installed on {} but not declared",
                events.into_iter().collect::<Vec<_>>().join(", ")
            ))
        })
        .collect()
}

fn commands(entries: &Value, nested: bool) -> impl Iterator<Item = &str> {
    let entries = entries.as_array().into_iter().flatten();
    let handlers: Box<dyn Iterator<Item = &Value>> = if nested {
        Box::new(
            entries
                .filter_map(|group| group.get("hooks").and_then(Value::as_array))
                .flatten(),
        )
    } else {
        Box::new(entries)
    };
    handlers
        .filter(|handler| {
            matches!(
                handler.get("type").and_then(Value::as_str),
                None | Some("command")
            )
        })
        .filter_map(|handler| handler.get("command").and_then(Value::as_str))
}
