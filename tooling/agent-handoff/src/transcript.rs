use crate::HandoffError;
use serde_json::{Map, Value};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const RETAINED_TRANSCRIPT_LINE_COUNT: usize = 500;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Agent {
    ClaudeCode,
    Codex,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Usage {
    pub agent: Agent,
    pub used: u64,
    pub window: Option<u64>,
}

pub fn find_latest_usage(transcript: &str) -> Result<Usage, HandoffError> {
    let mut physical_lines: Vec<&str> = transcript.split('\n').collect();
    if physical_lines.last() == Some(&"") {
        physical_lines.pop();
    }
    let retained_start = physical_lines
        .len()
        .saturating_sub(RETAINED_TRANSCRIPT_LINE_COUNT);
    let mut latest = None;

    for (index, line) in physical_lines[retained_start..].iter().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let record: Value = serde_json::from_str(line).map_err(|_| {
            HandoffError::usage(format!(
                "malformed transcript JSON at retained line {}",
                index + 1
            ))
        })?;
        let Some(record) = record.as_object() else {
            continue;
        };
        if let Some(usage) = parse_claude_usage(record)? {
            latest = Some(usage);
        } else if let Some(usage) = parse_codex_usage(record)? {
            latest = Some(usage);
        }
    }

    latest.ok_or_else(|| HandoffError::usage("no supported usage record in transcript"))
}

fn parse_claude_usage(record: &Map<String, Value>) -> Result<Option<Usage>, HandoffError> {
    if record.get("type").and_then(Value::as_str) != Some("assistant") {
        return Ok(None);
    }
    match record.get("isSidechain") {
        None | Some(Value::Bool(false)) => {}
        Some(Value::Bool(true)) => return Ok(None),
        Some(_) => return Err(HandoffError::usage("invalid Claude isSidechain")),
    }
    let Some(usage) = record
        .get("message")
        .and_then(Value::as_object)
        .and_then(|message| message.get("usage"))
        .and_then(Value::as_object)
    else {
        return Ok(None);
    };
    let input = parse_token_count(usage.get("input_tokens"), "Claude input_tokens", None)?;
    let cache_read = parse_token_count(
        usage.get("cache_read_input_tokens"),
        "Claude cache_read_input_tokens",
        Some(0),
    )?;
    let cache_creation = parse_token_count(
        usage.get("cache_creation_input_tokens"),
        "Claude cache_creation_input_tokens",
        Some(0),
    )?;
    let used = input + cache_read + cache_creation;
    if used > MAX_SAFE_INTEGER {
        return Err(HandoffError::usage("invalid Claude token total"));
    }
    Ok(Some(Usage {
        agent: Agent::ClaudeCode,
        used,
        window: None,
    }))
}

fn parse_codex_usage(record: &Map<String, Value>) -> Result<Option<Usage>, HandoffError> {
    if record.get("type").and_then(Value::as_str) != Some("event_msg") {
        return Ok(None);
    }
    let Some(payload) = record.get("payload").and_then(Value::as_object) else {
        return Ok(None);
    };
    if payload.get("type").and_then(Value::as_str) != Some("token_count") {
        return Ok(None);
    }
    let Some(info) = payload.get("info").and_then(Value::as_object) else {
        return Ok(None);
    };
    let Some(last_usage) = info.get("last_token_usage").and_then(Value::as_object) else {
        return Ok(None);
    };
    let window = parse_token_count(
        info.get("model_context_window"),
        "Codex model_context_window",
        None,
    )?;
    if window == 0 {
        return Err(HandoffError::usage("invalid Codex model_context_window"));
    }
    let used = parse_token_count(last_usage.get("input_tokens"), "Codex input_tokens", None)?;
    Ok(Some(Usage {
        agent: Agent::Codex,
        used,
        window: Some(window),
    }))
}

fn parse_token_count(
    value: Option<&Value>,
    field: &str,
    fallback: Option<u64>,
) -> Result<u64, HandoffError> {
    if value.is_none()
        && let Some(fallback) = fallback
    {
        return Ok(fallback);
    }
    value
        .and_then(parse_safe_integer)
        .ok_or_else(|| HandoffError::usage(format!("invalid {field}")))
}

fn parse_safe_integer(value: &Value) -> Option<u64> {
    let Value::Number(number) = value else {
        return None;
    };
    if let Some(value) = number.as_u64() {
        return (value <= MAX_SAFE_INTEGER).then_some(value);
    }
    let value = number.as_f64()?;
    (value.is_finite() && value >= 0.0 && value.fract() == 0.0 && value <= MAX_SAFE_INTEGER as f64)
        .then_some(value as u64)
}
