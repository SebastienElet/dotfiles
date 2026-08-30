mod claude;
mod codex;
mod context;

use crate::RetrievalReport;
use clap::ValueEnum;
use serde::Deserialize;
use serde_json::Value;
use std::fmt::{self, Display};
use std::path::{Component, Path, PathBuf};

const MAX_HOOK_INPUT_BYTES: usize = 1024 * 1024;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum HookAgent {
    Codex,
    Claude,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HookRequest {
    pub query: String,
    pub cwd: PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HookErrorClass {
    Rejection,
    Unavailable,
}

#[derive(Debug, Eq, PartialEq)]
pub struct HookError {
    class: HookErrorClass,
    code: &'static str,
    field: &'static str,
}

impl HookError {
    pub const fn class(&self) -> HookErrorClass {
        self.class
    }

    pub const fn code(&self) -> &'static str {
        self.code
    }

    pub const fn field(&self) -> &'static str {
        self.field
    }

    const fn rejection(code: &'static str, field: &'static str) -> Self {
        Self {
            class: HookErrorClass::Rejection,
            code,
            field,
        }
    }

    const fn unavailable(code: &'static str, field: &'static str) -> Self {
        Self {
            class: HookErrorClass::Unavailable,
            code,
            field,
        }
    }
}

impl Display for HookError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code, self.field)
    }
}

impl std::error::Error for HookError {}

pub fn parse_hook_request(agent: HookAgent, bytes: &[u8]) -> Result<HookRequest, HookError> {
    if bytes.is_empty() {
        return Err(HookError::rejection("empty_stdin", "stdin"));
    }
    if bytes.len() > MAX_HOOK_INPUT_BYTES {
        return Err(HookError::rejection("input_too_large", "stdin"));
    }
    match agent {
        HookAgent::Codex => codex::parse(bytes),
        HookAgent::Claude => claude::parse(bytes),
    }
}

pub fn render_hook_response(
    agent: HookAgent,
    report: &RetrievalReport,
) -> Result<Vec<u8>, HookError> {
    let context = context::render(report)?;
    match agent {
        HookAgent::Codex => codex::render(context.as_deref()),
        HookAgent::Claude => claude::render(context.as_deref()),
    }
}

pub(super) fn parse_payload(
    event: &PayloadField,
    prompt: &PayloadField,
    cwd: &PayloadField,
) -> Result<HookRequest, HookError> {
    let event = required_string(event, "missing_hook_event", "invalid_hook_event", "event")?;
    if event != "UserPromptSubmit" {
        return Err(HookError::rejection("invalid_hook_event", "event"));
    }
    let query = required_string(prompt, "missing_hook_query", "invalid_hook_query", "query")?;
    let query = query.trim();
    if query.is_empty() {
        return Err(HookError::rejection("invalid_hook_query", "query"));
    }
    let cwd = required_string(cwd, "missing_hook_cwd", "invalid_hook_cwd", "cwd")?;
    Ok(HookRequest {
        query: query.to_owned(),
        cwd: normalized_cwd(cwd)?,
    })
}

pub(super) const fn invalid_payload() -> HookError {
    HookError::rejection("invalid_hook_payload", "payload")
}

pub(super) const fn output_unavailable() -> HookError {
    HookError::unavailable("output_unavailable", "stdout")
}

#[derive(Default)]
pub(super) enum PayloadField {
    #[default]
    Missing,
    Present(Value),
}

impl<'de> Deserialize<'de> for PayloadField {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Value::deserialize(deserializer).map(Self::Present)
    }
}

fn required_string<'a>(
    value: &'a PayloadField,
    missing_code: &'static str,
    invalid_code: &'static str,
    field: &'static str,
) -> Result<&'a str, HookError> {
    match value {
        PayloadField::Missing => Err(HookError::rejection(missing_code, field)),
        PayloadField::Present(Value::String(value)) => Ok(value),
        PayloadField::Present(_) => Err(HookError::rejection(invalid_code, field)),
    }
}

fn normalized_cwd(value: &str) -> Result<PathBuf, HookError> {
    if value.is_empty() || value.contains('\0') {
        return Err(HookError::rejection("invalid_hook_cwd", "cwd"));
    }
    let path = Path::new(value);
    if !path.is_absolute()
        || path
            .components()
            .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
    {
        return Err(HookError::rejection("invalid_hook_cwd", "cwd"));
    }
    Ok(path.components().collect())
}
