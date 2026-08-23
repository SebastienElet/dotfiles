use crate::diagnostic::State;
use std::ffi::OsStr;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl ColorMode {
    pub(super) fn enabled(self, stdout_is_terminal: bool, no_color: Option<&OsStr>) -> bool {
        match self {
            Self::Auto => stdout_is_terminal && no_color.is_none_or(|value| value.is_empty()),
            Self::Always => true,
            Self::Never => false,
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct Colorizer {
    enabled: bool,
}

impl Colorizer {
    pub(super) fn new(enabled: bool) -> Self {
        Self { enabled }
    }

    pub(super) fn paint(self, state: State, value: impl AsRef<str>) -> String {
        let value = value.as_ref();
        if !self.enabled {
            return value.to_owned();
        }
        let code = match state {
            State::Error => 31,
            State::Healthy => 32,
            State::Drift => 33,
            State::Unsupported => 36,
        };
        format!("\x1b[{code}m{value}\x1b[0m")
    }
}
