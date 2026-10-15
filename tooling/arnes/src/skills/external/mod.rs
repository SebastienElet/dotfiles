use crate::Roots;
use crate::diagnostic::Diagnostic;
use crate::manifest::{Agent, Manifest, Scope};

mod cache_inventory;
mod claude;
mod codex;
mod cursor;
mod manifest;
mod model;
mod system;

pub(super) fn outside_manifest(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    [Agent::Claude, Agent::Codex, Agent::Cursor]
        .into_iter()
        .filter(|candidate| agent.is_none_or(|agent| agent == *candidate))
        .flat_map(|agent| {
            [Scope::User, Scope::Project]
                .into_iter()
                .map(move |scope| (agent, scope))
        })
        .filter(|(_, candidate)| scope.is_none_or(|scope| scope == *candidate))
        .filter(|pair| !manifest.combinations().any(|declared| declared == *pair))
        .flat_map(|(agent, scope)| diagnose(roots, manifest, agent, scope))
        .collect()
}

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

pub(super) fn is_claude_skills_plugin(agent: Agent, path: &std::path::Path) -> bool {
    agent == Agent::Claude
        && path
            .parent()
            .is_some_and(|root| crate::files::paths::canonical_within(path, root).is_some())
        && path.join(".claude-plugin/plugin.json").is_file()
}
