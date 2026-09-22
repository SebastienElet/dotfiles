use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const VERSION: u32 = 5;
pub const PREDICATE: &str = "https://proof-integrity.local/receipt/v5";

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Classification {
    pub schema_version: u32,
    pub triggered: bool,
    pub categories: BTreeMap<String, Vec<String>>,
    pub matched_paths: Vec<String>,
    pub unmatched_paths: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct State {
    pub path: String,
    pub kind: String,
    pub digest: String,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Subject {
    pub repository: String,
    pub base_ref: String,
    pub head_ref: String,
    pub remote_url: String,
    pub base_sha: String,
    pub merge_base_sha: String,
    pub head_sha: String,
    pub head_tree_sha: String,
    pub branch: String,
    pub include_worktree: bool,
    pub dirty_status_digest: String,
    pub submodule_status_digest: String,
    pub changed_paths: Vec<String>,
    pub path_states: Vec<State>,
    pub input_states: Vec<State>,
}
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Epoch {
    pub schema_version: u32,
    pub repository: String,
    pub subject: Subject,
    pub subject_digest: String,
    pub policy_digest: String,
    pub classifier: Classification,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub schema_version: u32,
    pub predicate_type: String,
    pub subject_digest: String,
    pub policy_digest: String,
    pub verdict: String,
    pub auditor: Auditor,
    pub claims: Vec<Claim>,
    pub witnesses: Vec<Witness>,
    pub limitations: Vec<String>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Auditor {
    pub host: String,
    pub session_id: String,
    pub fresh_session: Affirmed,
    pub forked: Denied,
    pub persistent_memory: Capability,
    pub author_independent: Affirmed,
    pub independent_first_pass: Affirmed,
    pub sandbox_mode: String,
    pub write_tools_enabled: Capability,
    pub isolation: Option<Isolation>,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum Capability {
    Known(bool),
    Unknown,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Isolation {
    pub requirement: String,
    pub enforced: Capability,
    pub evidence: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Claim {
    pub id: String,
    pub impact: String,
    pub kind: String,
    pub rationale: String,
    #[serde(rename = "claim")]
    pub statement: String,
    pub source: String,
    pub enforcement: String,
    pub oracle: String,
    pub paths: Vec<String>,
    pub positive_evidence: Observation,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub origin: String,
    pub artifact: String,
    pub input_basis: String,
    pub environment: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Witness {
    pub claim_id: String,
    pub mutant_digest: String,
    pub mutant: Mutant,
    pub red_exit_code: i32,
    pub green_exit_code: i32,
    pub artifact_digest: String,
    pub evidence: Evidence,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mutant {
    pub description: String,
    pub targets: Vec<Target>,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub scope: String,
    pub path: String,
    pub before_content: String,
    pub after_content: String,
    pub before_digest: String,
    pub after_digest: String,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub provenance: Observation,
    pub command: String,
    pub expected_failure: String,
    pub red_exit_code: i32,
    pub red_stdout: String,
    pub red_stderr: String,
    pub green_exit_code: i32,
    pub green_stdout: String,
    pub green_stderr: String,
    pub targets_digest: String,
    pub injection_mode: String,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "bool", into = "bool")]
pub struct Affirmed;
impl TryFrom<bool> for Affirmed {
    type Error = &'static str;
    fn try_from(value: bool) -> Result<Self, Self::Error> {
        if value {
            Ok(Self)
        } else {
            Err("declaration must be true")
        }
    }
}
impl From<Affirmed> for bool {
    fn from(_: Affirmed) -> Self {
        true
    }
}
#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(try_from = "bool", into = "bool")]
pub struct Denied;
impl TryFrom<bool> for Denied {
    type Error = &'static str;
    fn try_from(value: bool) -> Result<Self, Self::Error> {
        if value {
            Err("declaration must be false")
        } else {
            Ok(Self)
        }
    }
}
impl From<Denied> for bool {
    fn from(_: Denied) -> Self {
        false
    }
}
