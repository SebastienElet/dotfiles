use serde::Serialize;
use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum State {
    Healthy,
    Drift,
    Unsupported,
    Error,
}

impl Display for State {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Healthy => "healthy",
            Self::Drift => "drift",
            Self::Unsupported => "unsupported",
            Self::Error => "error",
        })
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    pub resource: String,
    pub state: State,
    pub message: String,
}

impl Diagnostic {
    pub fn new(resource: impl Into<String>, state: State, message: impl Into<String>) -> Self {
        Self {
            resource: resource.into(),
            state,
            message: message.into(),
        }
    }
}

impl Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} {}: {}",
            self.state,
            human_field(&self.resource),
            human_field(&self.message)
        )
    }
}

fn human_field(value: &str) -> String {
    value.replace('\r', "\\r").replace('\n', "\\n")
}

#[derive(Debug, Default, Eq, PartialEq)]
pub struct Report {
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    pub fn new(diagnostics: Vec<Diagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    pub fn exit_code(&self) -> u8 {
        self.diagnostics
            .iter()
            .map(|diagnostic| match diagnostic.state {
                State::Error => 2,
                State::Drift => 1,
                State::Healthy | State::Unsupported => 0,
            })
            .max()
            .unwrap_or(0)
    }

    pub fn human(&self) -> String {
        self.diagnostics
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.diagnostics)
    }
}
