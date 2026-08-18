mod feedback;
mod finish;
mod io;
mod list;

use super::MeasureError;
use super::model::HookAgent;
use super::repository;
use super::store::Store;
use clap::{Args, ValueEnum};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::{Path, PathBuf};

pub use feedback::record as feedback;
pub use finish::record as finish;
pub use list::render as list;

#[derive(Args)]
pub struct ListArgs {
    #[arg(long, value_enum)]
    pub agent: Option<HookAgent>,
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

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ResultRecord {
    pub schema_version: u8,
    pub run_id: String,
    pub revision: u64,
    pub recorded_at_ms: u64,
    pub merge_ready: MergeReady,
    pub human_minutes: f64,
    pub human_edited_diff: bool,
    pub failure_reason: Option<String>,
    pub evidence: Vec<String>,
    pub regression: bool,
    pub invariants: Vec<String>,
}

pub(super) fn validate_result_record(
    result: &ResultRecord,
    run_id: &str,
) -> Result<(), MeasureError> {
    if result.schema_version != 1 || result.run_id != run_id {
        return Err(MeasureError::new(
            "managed result.json has an unexpected identity",
        ));
    }
    if result.revision == 0 {
        return Err(MeasureError::new(
            "managed result.json has an invalid revision",
        ));
    }
    if result.recorded_at_ms == 0
        || !result.human_minutes.is_finite()
        || result.human_minutes < 0.0
        || result.evidence.iter().any(|value| value.trim().is_empty())
        || result
            .invariants
            .iter()
            .any(|value| value.trim().is_empty())
    {
        return Err(MeasureError::new(
            "managed result.json has invalid measurement values",
        ));
    }
    let reason = result
        .failure_reason
        .as_deref()
        .is_some_and(|value| !value.trim().is_empty());
    match result.merge_ready {
        MergeReady::Fail if !reason => Err(MeasureError::new(
            "managed result.json requires a failure reason",
        )),
        MergeReady::Pass if result.failure_reason.is_some() => Err(MeasureError::new(
            "managed result.json forbids a failure reason for pass",
        )),
        MergeReady::Unjudgeable if result.evidence.is_empty() => Err(MeasureError::new(
            "managed result.json requires evidence for unjudgeable",
        )),
        _ => Ok(()),
    }
}

pub(super) fn open_store() -> Result<Store, MeasureError> {
    let observed = env::current_dir()?;
    let root = repository::root(&observed);
    let protected = repository::protected_roots(&observed, root.as_deref().map(Path::new));
    Store::open(&protected)
}

pub(super) fn open_run(store: &Store, run_id: &str) -> Result<PathBuf, MeasureError> {
    validate_run_id(run_id)?;
    let path = store.run_path(run_id);
    let metadata =
        std::fs::symlink_metadata(&path).map_err(|error| map_unknown_run(error, run_id))?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(MeasureError::new(format!(
            "managed run path is not a real directory: {}",
            path.display()
        )));
    }
    let run = super::store::validation::read_run(&path.join("run.json"))?;
    if run.schema_version != 1 || run.run_id != run_id || !valid_agent(&run.agent) {
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

fn map_unknown_run(error: std::io::Error, run_id: &str) -> MeasureError {
    if error.kind() == std::io::ErrorKind::NotFound {
        MeasureError::new(format!("unknown run: {run_id}"))
    } else {
        error.into()
    }
}

fn valid_agent(agent: &str) -> bool {
    matches!(agent, "codex" | "claude-code" | "cursor")
}
