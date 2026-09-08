use super::super::model::HookAgent;
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};

#[derive(Args)]
#[command(about = "Append an explicit PR verdict to the local measurement timeline")]
pub struct PrVerdictArgs {
    #[command(flatten)]
    pub(super) pr: PrIdentity,
    #[arg(
        long,
        help = "Full head SHA actually judged, never the current branch inferred later"
    )]
    pub(super) head_sha: String,
    #[arg(long, value_enum)]
    pub(super) agent: Option<HookAgent>,
    #[command(flatten)]
    pub(super) data: VerdictData,
}

#[derive(Args, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PrIdentity {
    #[arg(long, help = "Canonical lowercase forge hostname, such as github.com")]
    pub forge: String,
    #[arg(
        long,
        help = "Canonical repository path from the PR metadata, without host or .git"
    )]
    pub repository: String,
    #[arg(long)]
    pub pr_id: u64,
}

#[derive(Args, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct VerdictData {
    #[arg(long, value_enum)]
    pub verdict: Verdict,
    #[arg(long)]
    pub blocking_findings: u64,
    #[arg(long)]
    pub non_blocking_findings: u64,
    #[arg(long, help = "Number of rows in the changed-behavior ledger")]
    pub observable_behaviors: u64,
    #[arg(
        long,
        help = "Number of distinct missing-evidence items in the review inventory"
    )]
    pub evidence_gaps: u64,
}

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(super) enum Verdict {
    ChangesRequired,
    ApprovedWithReservations,
    Approved,
}

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum EventType {
    PrVerdict,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PrEventRecord {
    pub schema_version: u8,
    pub event_type: EventType,
    pub timestamp_ms: u64,
    pub pr: PrIdentity,
    pub head_sha: String,
    pub agent: Option<String>,
    pub operating_system: String,
    pub architecture: String,
    pub data: VerdictData,
}
