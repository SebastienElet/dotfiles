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
    #[serde(rename = "input_tokens")]
    input: f64,
    #[serde(rename = "cached_input_tokens")]
    cached_input: f64,
    #[serde(rename = "output_tokens")]
    output: f64,
}

#[derive(Deserialize)]
struct Item {
    id: String,
    #[serde(rename = "type")]
    kind: String,
}

/// # Errors
/// Rejects malformed events, failed turns, missing or repeated completion events, and invalid token usage.
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
                let input = crate::numbers::safe_integer(usage.input);
                let cached_input = crate::numbers::safe_integer(usage.cached_input);
                let output = crate::numbers::safe_integer(usage.output);
                let (Some(input), Some(cached_input), Some(output)) = (input, cached_input, output)
                else {
                    return Err("Usage exceeds safe integer range".into());
                };
                let validated = Tokens {
                    input,
                    cached_input,
                    output,
                };
                tokens = Some(validated);
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
