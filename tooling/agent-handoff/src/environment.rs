use std::ffi::{OsStr, OsString};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Environment {
    pub handoff_token_threshold: Option<String>,
    pub claude_code_auto_compact_window: Option<String>,
    pub xdg_state_home: Option<String>,
    pub home: Option<String>,
}

impl Environment {
    #[allow(clippy::should_implement_trait)]
    pub fn from_iter(values: impl IntoIterator<Item = (OsString, OsString)>) -> Self {
        let mut environment = Self::default();

        for (name, value) in values {
            let value = Some(value.to_string_lossy().into_owned());
            match name.as_os_str() {
                name if name == OsStr::new("HANDOFF_TOKEN_THRESHOLD") => {
                    environment.handoff_token_threshold = value;
                }
                name if name == OsStr::new("CLAUDE_CODE_AUTO_COMPACT_WINDOW") => {
                    environment.claude_code_auto_compact_window = value;
                }
                name if name == OsStr::new("XDG_STATE_HOME") => {
                    environment.xdg_state_home = value;
                }
                name if name == OsStr::new("HOME") => {
                    environment.home = value;
                }
                _ => {}
            }
        }

        environment
    }

    pub fn current() -> Self {
        Self::from_iter(
            [
                "HANDOFF_TOKEN_THRESHOLD",
                "CLAUDE_CODE_AUTO_COMPACT_WINDOW",
                "XDG_STATE_HOME",
                "HOME",
            ]
            .into_iter()
            .filter_map(|name| std::env::var_os(name).map(|value| (OsString::from(name), value))),
        )
    }
}
