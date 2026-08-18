mod events;
mod fingerprint;
mod hook;
mod input;
mod install;
mod json;
mod model;
mod redaction;
mod repository;
mod result;
mod run;
mod store;

use std::fmt::{self, Display};

pub use hook::capture;
pub use install::{InstallHooksArgs, install_hooks};
pub use model::HookAgent;
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

impl From<serde_json::Error> for MeasureError {
    fn from(error: serde_json::Error) -> Self {
        Self(error.to_string())
    }
}
