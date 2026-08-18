use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub enum HookAgent {
    Codex,
    ClaudeCode,
    Cursor,
}

impl HookAgent {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Codex => "codex",
            Self::ClaudeCode => "claude-code",
            Self::Cursor => "cursor",
        }
    }

    pub fn session_key(self) -> &'static str {
        match self {
            Self::Codex | Self::ClaudeCode => "session_id",
            Self::Cursor => "conversation_id",
        }
    }
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    pub schema_version: u8,
    pub run_id: String,
    pub agent: String,
    pub session_id: String,
    pub started_at_ms: u64,
    pub model: Option<String>,
    pub repository: Option<RepositoryRecord>,
    pub harness_fingerprint: String,
    pub harness_fingerprint_limitations: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RepositoryRecord {
    pub root: String,
    pub head: Option<String>,
    pub branch: Option<String>,
    pub dirty: bool,
}

#[derive(Serialize)]
pub struct EventRecord {
    pub timestamp_ms: u64,
    pub event_id: String,
    pub event: String,
    pub native_event: String,
    pub artifact: String,
    pub native_ids: Map<String, Value>,
}

#[derive(Serialize)]
pub struct PromptRecord {
    pub timestamp_ms: u64,
    pub event_id: String,
    pub session_id: String,
    pub prompt_id: Option<String>,
    pub prompt: String,
}

#[derive(Serialize)]
pub struct InvalidRecord {
    pub timestamp_ms: u64,
    pub agent: &'static str,
    pub size: usize,
    pub sha256: String,
    pub error: String,
}
