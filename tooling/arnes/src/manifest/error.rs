use std::fmt::{self, Display};

#[derive(Debug)]
pub struct ManifestError {
    field: String,
    reason: String,
}

impl ManifestError {
    pub(super) fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.reason)
    }
}

impl std::error::Error for ManifestError {}
