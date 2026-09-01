use super::observed::ObservedConfiguration;
use crate::Roots;
use crate::files::paths::canonical_within;
use crate::manifest::{Agent, Scope};
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};

mod agent_json;
mod codex;

#[derive(Debug)]
pub(super) struct ConfigurationError(String);

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

pub(super) fn load(
    roots: &Roots,
    agent: Agent,
    scope: Scope,
    managed_names: &[&str],
) -> Result<Option<ObservedConfiguration>, ConfigurationError> {
    let (path, root) = configuration_path(roots, agent, scope);
    let Some(bytes) = read_optional(&path, root)? else {
        return Ok(None);
    };
    match agent {
        Agent::Claude => {
            agent_json::load(roots, &bytes, true, scope == Scope::Project, managed_names)
        }
        Agent::Cursor => agent_json::load(roots, &bytes, false, false, managed_names),
        Agent::Codex => codex::load(&bytes, managed_names),
    }
}

fn configuration_path(roots: &Roots, agent: Agent, scope: Scope) -> (PathBuf, &Path) {
    let root = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let relative = match (agent, scope) {
        (Agent::Claude, Scope::User) => Path::new(".claude.json"),
        (Agent::Claude, Scope::Project) => Path::new(".mcp.json"),
        (Agent::Cursor, _) => Path::new(".cursor/mcp.json"),
        (Agent::Codex, _) => Path::new(".codex/config.toml"),
    };
    (root.join(relative), root)
}

fn read_optional(path: &Path, root: &Path) -> Result<Option<Vec<u8>>, ConfigurationError> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Ok(_) if canonical_within(path, root).is_none() => {
            return Err(ConfigurationError::new(format!(
                "{} escapes its scope root",
                path.display()
            )));
        }
        Err(_) | Ok(_) => {}
    }
    match fs::read(path) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(_) => Err(ConfigurationError::new(format!(
            "could not read {}",
            path.display()
        ))),
    }
}

#[cfg(test)]
mod tests;
