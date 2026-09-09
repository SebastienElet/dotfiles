use serde::Deserialize;
use std::collections::BTreeSet;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tokens {
    pub input: u64,
    pub cached_input: u64,
    pub output: u64,
}

#[derive(Debug)]
pub struct CodexMetrics {
    pub tokens: Tokens,
    pub tool_calls: u64,
}

#[derive(Deserialize)]
struct Event {
    #[serde(rename = "type")]
    kind: String,
    #[serde(flatten)]
    fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: f64,
    cached_input_tokens: f64,
    output_tokens: f64,
}

#[derive(Deserialize)]
struct Item {
    id: String,
    #[serde(rename = "type")]
    kind: String,
}

pub fn parse_codex_events(output: &str) -> Result<CodexMetrics, String> {
    let mut tokens = None;
    let mut calls = BTreeSet::new();
    for line in output.split('\n').filter(|line| !line.is_empty()) {
        let event: Event = serde_json::from_str(line).map_err(|error| error.to_string())?;
        match event.kind.as_str() {
            "error" | "turn.failed" => return Err("Agent failure event".into()),
            "turn.completed" => {
                if tokens.is_some() {
                    return Err("One completed turn required".into());
                }
                let usage: Usage = event_field(&event, "usage")?;
                if [
                    usage.input_tokens,
                    usage.cached_input_tokens,
                    usage.output_tokens,
                ]
                .iter()
                .any(|value| {
                    !(0.0..=9_007_199_254_740_991.0).contains(value) || value.fract() != 0.0
                }) {
                    return Err("Usage exceeds safe integer range".into());
                }
                tokens = Some(Tokens {
                    input: usage.input_tokens as u64,
                    cached_input: usage.cached_input_tokens as u64,
                    output: usage.output_tokens as u64,
                });
            }
            "item.completed" => {
                let item: Item = event_field(&event, "item")?;
                if [
                    "command_execution",
                    "mcp_tool_call",
                    "web_search",
                    "file_change",
                ]
                .contains(&item.kind.as_str())
                {
                    calls.insert(item.id);
                }
            }
            _ => {}
        }
    }
    Ok(CodexMetrics {
        tokens: tokens.ok_or("One completed turn required")?,
        tool_calls: calls.len() as u64,
    })
}

fn event_field<T: serde::de::DeserializeOwned>(event: &Event, name: &str) -> Result<T, String> {
    let value = event
        .fields
        .get(name)
        .ok_or_else(|| format!("Missing {name}"))?;
    serde_json::from_value(value.clone()).map_err(|error| error.to_string())
}

#[cfg(test)]
mod tests;
