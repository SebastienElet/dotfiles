use crate::manifest::{Agent, UserConfig};
use serde_json::{Value, json};

pub(super) fn mismatches(agent: Agent, config: &UserConfig, actual: &Value) -> Vec<String> {
    let mut expected = vec![("model", json!(config.model))];
    match agent {
        Agent::Claude => {
            push(&mut expected, "effortLevel", config.effort.as_ref());
            push(
                &mut expected,
                "autoCompactWindow",
                config.auto_compact_window,
            );
        }
        Agent::Cursor => {
            expected[0].0 = "model.modelId";
            push(&mut expected, "maxMode", config.max_mode);
        }
        Agent::Codex => {
            push(
                &mut expected,
                "model_reasoning_effort",
                config.effort.as_ref(),
            );
            push(&mut expected, "model_context_window", config.context_window);
            push(
                &mut expected,
                "model_auto_compact_token_limit",
                config.auto_compact_window,
            );
        }
    }
    expected
        .into_iter()
        .filter_map(|(path, expected)| mismatch(actual, path, expected))
        .collect()
}

fn push<T: serde::Serialize>(
    expected: &mut Vec<(&'static str, Value)>,
    path: &'static str,
    value: Option<T>,
) {
    if let Some(value) = value {
        expected.push((path, json!(value)));
    }
}

fn mismatch(actual: &Value, path: &str, expected: Value) -> Option<String> {
    match lookup(actual, path) {
        Some(actual) if actual == &expected => None,
        Some(actual) => Some(format!(
            "{path} is {} (expected {})",
            display(actual),
            display(&expected)
        )),
        None => Some(format!(
            "{path} is missing (expected {})",
            display(&expected)
        )),
    }
}

fn lookup<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.')
        .try_fold(value, |value, part| value.get(part))
}

fn display(value: &Value) -> String {
    serde_json::to_string(value).expect("JSON values serialize")
}
