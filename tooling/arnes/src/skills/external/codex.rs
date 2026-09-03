use super::model::{Exposure, Plugin, Topology, plugin_diagnostics};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Scope};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

mod command;
mod config;
mod resolution;
mod state;

pub(super) fn diagnose(roots: &Roots, policy: &Manifest, scope: Scope) -> Vec<Diagnostic> {
    if scope == Scope::Project {
        return Vec::new();
    }
    let path = roots.home().join(".codex/config.toml");
    let config = match config::load(&path, roots.home()) {
        Ok(config) => config,
        Err(detail) => {
            return vec![Diagnostic::new(
                "skills",
                State::Error,
                format!(
                    "external codex user plugin configuration origin=plugin ownership=external exposure=unknown topology=unreadable policy=unknown activation=unknown path={} detail={detail}",
                    path.display(),
                ),
            )];
        }
    };
    let resolved = state::load(roots.home());
    if config.plugins.is_empty()
        && let Err(detail) = &resolved
    {
        return vec![resolver_failure_diagnostic(detail)];
    }
    let ids = configured_and_resolved_ids(&config.plugins, resolved.as_ref().ok());
    let plugins = ids
        .into_iter()
        .map(|id| {
            let enabled = config.plugins.get(&id).and_then(|plugin| plugin.enabled);
            match &resolved {
                Ok(state) => resolution::resolve(roots.home(), id, enabled, state),
                Err(detail) => resolution::unresolved(id, enabled, detail.clone()),
            }
        })
        .collect::<Vec<_>>();
    let unresolved = resolution_diagnostic(&plugins);
    let mut diagnostics = plugin_diagnostics(policy, Agent::Codex, scope, plugins);
    diagnostics.extend(unresolved);
    diagnostics
}

fn configured_and_resolved_ids(
    configured: &BTreeMap<String, config::PluginConfig>,
    resolved: Option<&state::ResolverState>,
) -> BTreeSet<String> {
    configured
        .keys()
        .cloned()
        .chain(
            resolved
                .into_iter()
                .flat_map(|state| state.plugins.iter().map(|plugin| plugin.plugin_id.clone())),
        )
        .collect()
}

fn resolution_diagnostic(plugins: &[Plugin]) -> Option<Diagnostic> {
    let failures = plugins
        .iter()
        .filter(|plugin| plugin.topology == Topology::Unknown)
        .map(|plugin| {
            format!(
                "{}: {}",
                plugin.id,
                plugin
                    .detail
                    .as_deref()
                    .unwrap_or("unknown resolver failure")
            )
        })
        .collect::<Vec<_>>();
    (!failures.is_empty()).then(|| resolver_failure_diagnostic(&failures.join("; ")))
}

fn resolver_failure_diagnostic(detail: &str) -> Diagnostic {
    Diagnostic::new(
        "skills",
        State::Unsupported,
        format!(
            "external codex user plugin resolution origin=plugin ownership=external exposure=unknown topology=unknown policy=unknown activation=unknown detail={detail}"
        ),
    )
}

pub(super) fn skill_exposure(roots: &Roots, skill_file: &Path) -> Exposure {
    let Ok(config) = config::load(&roots.home().join(".codex/config.toml"), roots.home()) else {
        return Exposure::Unknown;
    };
    config.skill_exposure(skill_file)
}
