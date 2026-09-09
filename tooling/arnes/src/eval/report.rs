use super::{
    contracts::{BehavioralCase, Observation},
    sources::ResolvedSource,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Status {
    Pass,
    Fail,
    Invalid,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunError {
    AgentFailed,
    Timeout,
    ProtocolInvalid,
    ObservationInvalid,
    OutputLimit,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Tokens {
    pub input: u64,
    pub cached_input: u64,
    pub output: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Run {
    pub status: Status,
    #[serde(deserialize_with = "required_nullable")]
    pub error: Option<RunError>,
    pub observations: Vec<Observation>,
    #[serde(deserialize_with = "required_nullable")]
    pub tokens: Option<Tokens>,
    #[serde(deserialize_with = "required_nullable")]
    pub tool_calls: Option<u64>,
    #[serde(deserialize_with = "required_nullable")]
    pub duration_ms: Option<f64>,
}

fn required_nullable<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    deserializer: D,
) -> Result<Option<T>, D::Error> {
    Option::deserialize(deserializer)
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Controls {
    pub sandbox: String,
    pub network: bool,
    pub tools: String,
    pub timeout_seconds: u64,
    pub reasoning_effort: String,
    pub token_budget: (),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Harness {
    pub git_revision: String,
    pub instruction_fingerprint: String,
    pub skill_fingerprint: String,
    pub variant: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Environment {
    Rust {
        platform: String,
        architecture: String,
        runtime: String,
    },
    Legacy {
        platform: String,
        architecture: String,
        bun: String,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ReportCase {
    pub definition: BehavioralCase,
    pub prompt: String,
    pub prompt_fingerprint: String,
    pub sources: Vec<ResolvedSource>,
    pub fixture_revision: String,
    pub runs: Vec<Run>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Report {
    pub schema_version: u64,
    pub agent: String,
    pub agent_version: String,
    pub model: String,
    pub date: String,
    pub harness: Harness,
    pub runner_revision: String,
    pub environment: Environment,
    pub controls: Controls,
    pub run_count: usize,
    pub cases: Vec<ReportCase>,
    pub limitations: Vec<String>,
}
