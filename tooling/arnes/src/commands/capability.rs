use crate::manifest::{Agent, Scope};
use std::path::{Path, PathBuf};

pub(super) fn destination(agent: Agent, scope: Scope, name: &str) -> Option<PathBuf> {
    match (agent, scope) {
        (Agent::Claude, Scope::User | Scope::Project) => {
            Some(Path::new(".claude/commands").join(format!("{name}.md")))
        }
        _ => None,
    }
}
