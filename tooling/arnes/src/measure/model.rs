use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Serialize)]
#[serde(untagged)]
pub enum RunRecord {
    V1(RunRecordV1),
    V2(RunRecordV2),
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecordV1 {
    schema_version: u8,
    run_id: String,
    agent: String,
    session_id: String,
    started_at_ms: u64,
    model: Option<String>,
    repository: Option<String>,
    repository_commit: Option<String>,
    repository_branch: Option<String>,
    repository_dirty: Option<bool>,
    harness_fingerprint: String,
    harness_fingerprint_limitations: Vec<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecordV2 {
    pub(super) schema_version: u8,
    pub(super) run_id: String,
    pub(super) agent: String,
    pub(super) started_at_ms: u64,
    pub(super) model_fingerprint: Option<String>,
    pub(super) repository_commit: Option<String>,
    pub(super) repository_dirty: Option<bool>,
    pub(super) harness_fingerprint: String,
    pub(super) harness_fingerprint_limitations: Vec<String>,
    pub(super) operating_system: String,
    pub(super) architecture: String,
}

impl RunRecord {
    pub fn parse(value: Value) -> Result<Self, serde_json::Error> {
        match value.get("schema_version").and_then(Value::as_u64) {
            Some(1) => serde_json::from_value(value).map(Self::V1),
            Some(2) => serde_json::from_value(value).map(Self::V2),
            _ => serde_json::from_value::<RunRecordV2>(value).map(Self::V2),
        }
    }

    pub fn schema_version(&self) -> u8 {
        match self {
            Self::V1(run) => run.schema_version,
            Self::V2(run) => run.schema_version,
        }
    }

    pub fn run_id(&self) -> &str {
        match self {
            Self::V1(run) => &run.run_id,
            Self::V2(run) => &run.run_id,
        }
    }

    pub fn agent(&self) -> &str {
        match self {
            Self::V1(run) => &run.agent,
            Self::V2(run) => &run.agent,
        }
    }

    pub fn session_id(&self) -> Option<&str> {
        match self {
            Self::V1(run) => Some(&run.session_id),
            Self::V2(_) => None,
        }
    }

    pub fn started_at_ms(&self) -> u64 {
        match self {
            Self::V1(run) => run.started_at_ms,
            Self::V2(run) => run.started_at_ms,
        }
    }

    pub fn repository(&self) -> Option<&str> {
        match self {
            Self::V1(run) => run.repository.as_deref(),
            Self::V2(_) => None,
        }
    }
}

pub struct RepositoryRecord {
    pub head: Option<String>,
    pub dirty: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventRecord {
    pub schema_version: u8,
    pub timestamp_ms: u64,
    pub event: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
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
