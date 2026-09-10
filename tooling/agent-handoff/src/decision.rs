use crate::{Agent, Environment, HandoffError, Usage};
use serde::Serialize;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
const TOKENS_PER_THOUSAND: u64 = 1_000;

#[derive(Serialize)]
struct BlockDecision<'a> {
    decision: &'static str,
    reason: &'a str,
}

/// # Errors
/// Returns a usage error when the selected threshold or context window is missing or invalid.
pub fn select_threshold(usage: &Usage, environment: &Environment) -> Result<u64, HandoffError> {
    if let Some(value) = environment
        .handoff_token_threshold
        .as_deref()
        .filter(|value| !value.is_empty())
    {
        return parse_positive_integer(value, "HANDOFF_TOKEN_THRESHOLD");
    }

    let window = if let Some(window) = usage.window {
        if window == 0 || window > MAX_SAFE_INTEGER {
            return Err(HandoffError::usage("invalid Codex model_context_window"));
        }
        window
    } else {
        let value = environment
            .claude_code_auto_compact_window
            .as_deref()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| HandoffError::usage("missing context window"))?;
        parse_positive_integer(value, "CLAUDE_CODE_AUTO_COMPACT_WINDOW")?
    };

    Ok((window * 85).div_euclid(100))
}

/// # Errors
/// Returns an unexpected error when the hook decision cannot be serialized.
pub fn handoff_output(usage: &Usage, threshold: u64) -> Result<Vec<u8>, HandoffError> {
    let invocation = match usage.agent {
        Agent::ClaudeCode => "/handoff",
        Agent::Codex => "$handoff",
    };
    let reason = format!(
        "Context is at {}k tokens, past the {}k handoff threshold. Start no new work. Use {invocation} to emit the resume prompt for a fresh session, then stop.",
        usage.used.div_euclid(TOKENS_PER_THOUSAND),
        threshold.div_euclid(TOKENS_PER_THOUSAND),
    );
    let mut output = serde_json::to_vec_pretty(&BlockDecision {
        decision: "block",
        reason: &reason,
    })
    .map_err(|_| HandoffError::unexpected("cannot serialize handoff decision"))?;
    output.push(b'\n');
    Ok(output)
}

fn parse_positive_integer(value: &str, name: &str) -> Result<u64, HandoffError> {
    let valid_decimal = value.as_bytes().split_first().is_some_and(|(first, rest)| {
        matches!(first, b'1'..=b'9') && rest.iter().all(u8::is_ascii_digit)
    });
    if !valid_decimal {
        return Err(HandoffError::usage(format!("invalid {name}")));
    }
    value
        .parse::<u64>()
        .ok()
        .filter(|value| *value <= MAX_SAFE_INTEGER)
        .ok_or_else(|| HandoffError::usage(format!("invalid {name}")))
}
