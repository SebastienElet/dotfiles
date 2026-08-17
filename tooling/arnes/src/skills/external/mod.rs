use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Scope};

mod claude;
mod codex;
mod cursor;
mod manifest;
mod model;
mod system;

pub(super) fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Vec<Diagnostic> {
    let mut diagnostics = system::diagnose(roots, manifest, agent, scope);
    diagnostics.extend(match agent {
        Agent::Claude => claude::diagnose(roots, manifest, scope),
        Agent::Cursor => cursor::diagnose(roots, manifest, scope),
        Agent::Codex => codex::diagnose(roots, manifest, scope),
    });
    diagnostics
}

pub(super) fn unsupported(agent: Agent, scope: Scope, subject: &str) -> Diagnostic {
    Diagnostic::new(
        "skills",
        State::Unsupported,
        format!(
            "external {agent} {scope} {subject} origin=system ownership=external exposure=unknown topology=unknown policy=unknown activation=unknown"
        ),
    )
}

pub(super) fn is_claude_skills_plugin(agent: Agent, path: &std::path::Path) -> bool {
    agent == Agent::Claude
        && path
            .parent()
            .is_some_and(|root| super::paths::canonical_within(path, root).is_some())
        && path.join(".claude-plugin/plugin.json").is_file()
}
