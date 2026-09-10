use std::ffi::{OsStr, OsString};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Environment {
    pub handoff_token_threshold: Option<String>,
    pub claude_code_auto_compact_window: Option<String>,
    pub xdg_state_home: Option<OsString>,
    pub home: Option<OsString>,
}

impl FromIterator<(OsString, OsString)> for Environment {
    fn from_iter<T: IntoIterator<Item = (OsString, OsString)>>(values: T) -> Self {
        let mut environment = Self::default();

        for (name, value) in values {
            match name.as_os_str() {
                name if name == OsStr::new("HANDOFF_TOKEN_THRESHOLD") => {
                    environment.handoff_token_threshold =
                        Some(value.to_string_lossy().into_owned());
                }
                name if name == OsStr::new("CLAUDE_CODE_AUTO_COMPACT_WINDOW") => {
                    environment.claude_code_auto_compact_window =
                        Some(value.to_string_lossy().into_owned());
                }
                name if name == OsStr::new("XDG_STATE_HOME") => {
                    environment.xdg_state_home = Some(value);
                }
                name if name == OsStr::new("HOME") => {
                    environment.home = Some(value);
                }
                _ => {}
            }
        }

        environment
    }
}

impl Environment {
    #[must_use]
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
