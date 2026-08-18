use crate::manifest::{Agent, Scope};
use std::path::{Path, PathBuf};

pub(super) fn destination(agent: Agent, scope: Scope, name: &str) -> Option<PathBuf> {
    supported(agent, scope).then(|| Path::new(".claude/commands").join(format!("{name}.md")))
}

pub(super) fn supported(agent: Agent, scope: Scope) -> bool {
    matches!(
        (agent, scope),
        (Agent::Claude, Scope::User | Scope::Project)
    )
}
