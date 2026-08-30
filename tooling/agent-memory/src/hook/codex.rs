use super::{
    HookError, HookRequest, PayloadField, invalid_payload, output_unavailable, parse_payload,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct CodexPayload {
    #[serde(default)]
    hook_event_name: PayloadField,
    #[serde(default)]
    prompt: PayloadField,
    #[serde(default)]
    cwd: PayloadField,
}

pub(super) fn parse(bytes: &[u8]) -> Result<HookRequest, HookError> {
    let payload: CodexPayload = serde_json::from_slice(bytes).map_err(|_| invalid_payload())?;
    parse_payload(&payload.hook_event_name, &payload.prompt, &payload.cwd)
}

pub(super) fn render(context: Option<&str>) -> Result<Vec<u8>, HookError> {
    match context {
        Some(additional_context) => serde_json::to_vec(&CodexResponse {
            hook_specific_output: CodexOutput {
                hook_event_name: "UserPromptSubmit",
                additional_context,
            },
        })
        .map_err(|_| output_unavailable()),
        None => Ok(b"{}".to_vec()),
    }
}

#[derive(Serialize)]
struct CodexResponse<'a> {
    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: CodexOutput<'a>,
}

#[derive(Serialize)]
struct CodexOutput<'a> {
    #[serde(rename = "hookEventName")]
    hook_event_name: &'static str,
    #[serde(rename = "additionalContext")]
    additional_context: &'a str,
}
