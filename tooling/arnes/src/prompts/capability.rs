use crate::manifest::{Agent, Scope};
use std::path::Path;

pub fn registry(agent: Agent, scope: Scope) -> Option<&'static Path> {
    match (agent, scope) {
        (Agent::Claude, Scope::User | Scope::Project) => Some(Path::new(".claude/commands")),
        (Agent::Cursor, Scope::Project) => Some(Path::new(".cursor/commands")),
        _ => None,
    }
}
