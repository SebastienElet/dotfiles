use crate::Roots;
use crate::files::paths::canonical_within;
use crate::manifest::Scope;
use std::fmt::{self, Display};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

#[derive(Debug)]
pub struct ConfigurationError(String);

impl ConfigurationError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for ConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

pub fn load(roots: &Roots, scope: Scope) -> Result<Option<Vec<String>>, ConfigurationError> {
    let root = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let path = root.join(".codex/config.toml");
    let Some(bytes) = read_optional(&path, root)? else {
        return Ok(None);
    };
    let input = std::str::from_utf8(&bytes)
        .map_err(|_| ConfigurationError::new("Codex configuration is not UTF-8"))?;
    let value = toml::from_str::<toml::Value>(input)
        .map_err(|_| ConfigurationError::new("Codex configuration is malformed"))?;
    let Some(tui) = value.get("tui") else {
        return Ok(None);
    };
    let tui = tui
        .as_table()
        .ok_or_else(|| ConfigurationError::new("tui must be a table"))?;
    let Some(status_line) = tui.get("status_line") else {
        return Ok(None);
    };
    status_line
        .as_array()
        .ok_or_else(|| ConfigurationError::new("tui.status_line must be an array of strings"))?
        .iter()
        .map(|item| {
            item.as_str().map(str::to_owned).ok_or_else(|| {
                ConfigurationError::new("tui.status_line must be an array of strings")
            })
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Some)
}

fn read_optional(path: &Path, root: &Path) -> Result<Option<Vec<u8>>, ConfigurationError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Ok(_) if canonical_within(path, root).is_none() => {
            return Err(ConfigurationError::new(format!(
                "{} escapes its scope root",
                path.display()
            )));
        }
        Err(_) | Ok(_) => {}
    }
    fs::read(path)
        .map(Some)
        .map_err(|_| ConfigurationError::new(format!("could not read {}", path.display())))
}
