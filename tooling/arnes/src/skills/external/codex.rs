use super::model::{Exposure, Plugin, Topology, plugin_diagnostics};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Scope};
use std::path::Path;

mod config;

const INVENTORY_UNAVAILABLE: &str = "active plugin inventory is unavailable in read-only Doctor; the external Codex resolver is not executed";

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
    let plugins = config
        .plugins
        .into_iter()
        .map(|(id, plugin)| configured_plugin(id, plugin.enabled))
        .collect();
    let mut diagnostics = plugin_diagnostics(policy, Agent::Codex, scope, plugins);
    diagnostics.push(Diagnostic::new(
        "skills",
        State::Unsupported,
        format!(
            "external codex user plugin resolution origin=plugin ownership=external exposure=unknown topology=unknown policy=unknown activation=unknown detail={INVENTORY_UNAVAILABLE}"
        ),
    ));
    diagnostics
}

fn configured_plugin(id: String, enabled: Option<bool>) -> Plugin {
    Plugin {
        id,
        artifact: None,
        version: None,
        path: None,
        exposure: match enabled {
            Some(true) => Exposure::Enabled,
            Some(false) => Exposure::Disabled,
            None => Exposure::Unknown,
        },
        topology: Topology::Unknown,
        detail: Some(INVENTORY_UNAVAILABLE.to_owned()),
        skills: Vec::new(),
    }
}

pub(super) fn skill_exposure(roots: &Roots, skill_file: &Path) -> Exposure {
    let Ok(config) = config::load(&roots.home().join(".codex/config.toml"), roots.home()) else {
        return Exposure::Unknown;
    };
    config.skill_exposure(skill_file)
}
