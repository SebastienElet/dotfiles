use super::{
    HookError, HookRequest, PayloadField, invalid_payload, output_unavailable, parse_payload,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct ClaudePayload {
    #[serde(default)]
    hook_event_name: PayloadField,
    #[serde(default)]
    prompt: PayloadField,
    #[serde(default)]
    cwd: PayloadField,
}

pub(super) fn parse(bytes: &[u8]) -> Result<HookRequest, HookError> {
    let payload: ClaudePayload = serde_json::from_slice(bytes).map_err(|_| invalid_payload())?;
    parse_payload(&payload.hook_event_name, &payload.prompt, &payload.cwd)
}

pub(super) fn render(context: Option<&str>) -> Result<Vec<u8>, HookError> {
    context.map_or_else(
        || Ok(b"{}".to_vec()),
        |additional_context| {
            serde_json::to_vec(&ClaudeResponse {
                hook_specific_output: ClaudeOutput {
                    hook_event_name: "UserPromptSubmit",
                    additional_context,
                },
            })
            .map_err(|_| output_unavailable())
        },
    )
}

#[derive(Serialize)]
struct ClaudeResponse<'a> {
    #[serde(rename = "hookSpecificOutput")]
    hook_specific_output: ClaudeOutput<'a>,
}

#[derive(Serialize)]
struct ClaudeOutput<'a> {
    #[serde(rename = "hookEventName")]
    hook_event_name: &'static str,
    #[serde(rename = "additionalContext")]
    additional_context: &'a str,
}
