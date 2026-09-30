use crate::manifest::{Agent, Scope};
use std::path::{Path, PathBuf};

pub fn destination(agent: Agent, scope: Scope, name: &str) -> Option<PathBuf> {
    supported(agent, scope).then(|| Path::new(".claude/commands").join(format!("{name}.md")))
}

pub const fn supported(agent: Agent, scope: Scope) -> bool {
    matches!(
        (agent, scope),
        (Agent::Claude, Scope::User | Scope::Project)
    )
}
