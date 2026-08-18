use super::model::{Exposure, Plugin, Topology, plugin_diagnostics};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within};
use crate::manifest::{Agent, Manifest, Scope};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

mod command;
mod resolution;
mod state;

#[derive(Default, Deserialize)]
struct Config {
    #[serde(default)]
    plugins: BTreeMap<String, PluginConfig>,
    #[serde(default)]
    skills: SkillsConfig,
}

#[derive(Deserialize)]
struct PluginConfig {
    enabled: Option<bool>,
}

#[derive(Default, Deserialize)]
struct SkillsConfig {
    #[serde(default)]
    config: Vec<SkillConfig>,
}

#[derive(Deserialize)]
struct SkillConfig {
    path: PathBuf,
    enabled: bool,
}

pub(super) fn diagnose(roots: &Roots, policy: &Manifest, scope: Scope) -> Vec<Diagnostic> {
    if scope == Scope::Project {
        return vec![Diagnostic::new(
            "skills",
            State::Unsupported,
            "external codex project plugin inventory origin=plugin ownership=external exposure=unknown topology=unknown policy=unknown activation=unknown detail=project plugin activation has no documented filesystem registry",
        )];
    }
    let path = roots.home().join(".codex/config.toml");
    let config = match load(&path, roots.home()) {
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
    configured: &BTreeMap<String, PluginConfig>,
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
    let Ok(config) = load(&roots.home().join(".codex/config.toml"), roots.home()) else {
        return Exposure::Unknown;
    };
    let matching = config
        .skills
        .config
        .iter()
        .filter(|entry| same_path(&entry.path, skill_file))
        .map(|entry| entry.enabled)
        .collect::<Vec<_>>();
    match matching.as_slice() {
        [] | [true] => Exposure::Enabled,
        [false] => Exposure::Disabled,
        _ => Exposure::Unknown,
    }
}

fn load(path: &Path, root: &Path) -> Result<Config, &'static str> {
    match fs::symlink_metadata(path) {
        Ok(_) if canonical_within(path, root).is_none() => {
            return Err("config resolves outside HOME");
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return if ancestor_within(path, root) {
                Ok(Config::default())
            } else {
                Err("config parent resolves outside HOME")
            };
        }
        Err(_) => return Err("config metadata could not be read"),
    }
    let contents = fs::read_to_string(path).map_err(|_| "config could not be read")?;
    toml::from_str(&contents).map_err(|_| "config is invalid TOML")
}

fn same_path(left: &Path, right: &Path) -> bool {
    left == right
        || fs::canonicalize(left)
            .ok()
            .zip(fs::canonicalize(right).ok())
            .is_some_and(|(left, right)| left == right)
}
