mod events;
mod fingerprint;
mod hook;
mod input;
mod json;
mod model;
mod outcome;
mod report;
mod repository;
mod result;
mod retention;
mod run;
mod store;

use std::fmt::{self, Display};

pub use hook::capture;
pub use model::HookAgent;
pub use outcome::{OutcomeArgs, OutcomeStatus, UnjudgeableReason, record as outcome};
pub use report::{ReportArgs, render as report};
pub use result::{
    Adjudication, FailureCategory, FeedbackArgs, FeedbackSource, FinishArgs, ListArgs, ListFormat,
    MergeReady, Resolution, Severity, feedback, finish, list,
};

#[derive(Debug)]
pub struct MeasureError(String);

impl MeasureError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for MeasureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for MeasureError {}

impl From<std::io::Error> for MeasureError {
    fn from(error: std::io::Error) -> Self {
        Self(error.to_string())
    }
}

impl From<rustix::io::Errno> for MeasureError {
    fn from(error: rustix::io::Errno) -> Self {
        std::io::Error::from(error).into()
    }
}

impl From<serde_json::Error> for MeasureError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}
