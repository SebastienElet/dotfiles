mod feedback;
mod finish;
mod io;
mod list;
mod records;

use super::MeasureError;
use super::model::HookAgent;
use super::repository;
use super::store::{ManagedPath, Store};
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

pub use feedback::record as feedback;
pub use finish::record as finish;
pub(super) use io::read_optional_json;
pub(super) use io::visit_jsonl_typed;
pub use list::render as list;
pub(super) use records::{EventHistory, read_events_for_list_with};
pub(super) use records::{ResultRecord, validate_result_record};

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, value_enum)]
    pub agent: Option<HookAgent>,
    #[arg(
        long,
        help = "Report only runs without structured result history; silence does not prove blockage, active time, or process state"
    )]
    pub without_result: bool,
    #[arg(long, value_enum, default_value_t)]
    pub format: ListFormat,
}

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum ListFormat {
    #[default]
    Human,
    Json,
}

#[derive(Args)]
pub struct FinishArgs {
    pub run_id: String,
    #[arg(long, value_enum)]
    pub merge_ready: MergeReady,
    #[arg(long, allow_hyphen_values = true)]
    pub human_minutes: f64,
    #[arg(long)]
    pub human_edited_diff: bool,
    #[arg(long)]
    pub failure_reason: Option<String>,
    #[arg(long)]
    pub evidence: Vec<String>,
    #[arg(long)]
    pub regression: bool,
    #[arg(long)]
    pub invariant: Vec<String>,
}

#[derive(Args)]
pub struct FeedbackArgs {
    pub run_id: String,
    #[arg(long, value_enum)]
    pub source_type: FeedbackSource,
    #[arg(long)]
    pub source_id: String,
    #[arg(long)]
    pub scope: String,
    #[arg(long)]
    pub observed: String,
    #[arg(long)]
    pub expected: String,
    #[arg(long)]
    pub evidence: Vec<String>,
    #[arg(long)]
    pub invariant: Vec<String>,
    #[arg(long, value_enum)]
    pub severity: Severity,
    #[arg(long, value_enum)]
    pub adjudication: Adjudication,
    #[arg(long, value_enum)]
    pub resolution: Resolution,
    #[arg(long, value_enum)]
    pub failure_category: FailureCategory,
}

macro_rules! string_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
        #[serde(rename_all = "kebab-case")]
        pub enum $name { $($variant),+ }
    };
}

string_enum!(MergeReady {
    Pass,
    Fail,
    Unjudgeable
});
string_enum!(FeedbackSource { Human, Harness });
string_enum!(Severity {
    Blocking,
    Major,
    Minor,
    Note
});
string_enum!(Adjudication {
    Pending,
    Confirmed,
    Rejected
});
string_enum!(Resolution {
    Open,
    Fixed,
    WontFix,
    Duplicate
});
string_enum!(FailureCategory {
    Requirements,
    Correctness,
    Tests,
    Security,
    Performance,
    Maintainability,
    Tooling,
    Communication,
    Other
});

pub(super) fn open_store() -> Result<Store, MeasureError> {
    let observed = env::current_dir()?;
    let root = repository::root(&observed);
    let protected = repository::protected_roots(&observed, root.as_deref().map(Path::new));
    Store::open(&protected)
}

pub(super) fn open_run(store: &Store, run_id: &str) -> Result<ManagedPath, MeasureError> {
    validate_run_id(run_id)?;
    let path = store.run_path(run_id);
    if !path.exists()? {
        return Err(MeasureError::new(format!("unknown run: {run_id}")));
    }
    path.open_directory()?;
    let run = super::store::validation::read_run(&path.join("run.json"))?;
    if !matches!(run.schema_version(), 1 | 2) || run.run_id() != run_id || !valid_agent(run.agent())
    {
        return Err(MeasureError::new(
            "managed run.json has an unexpected identity",
        ));
    }
    Ok(path)
}

fn validate_run_id(run_id: &str) -> Result<(), MeasureError> {
    if run_id.len() == 64
        && run_id
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Ok(());
    }
    Err(MeasureError::new(
        "run id must be exactly 64 lowercase hexadecimal characters",
    ))
}

fn valid_agent(agent: &str) -> bool {
    matches!(agent, "codex" | "claude-code" | "cursor")
}
