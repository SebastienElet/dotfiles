use super::model::{Exposure, Plugin, Topology, plugin_diagnostics};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within};
use crate::manifest::{Agent, Manifest, Scope};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

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
    let plugins = config
        .plugins
        .into_iter()
        .map(|(id, plugin)| Plugin {
            id,
            version: None,
            path: None,
            exposure: match plugin.enabled {
                Some(true) => Exposure::Enabled,
                Some(false) => Exposure::Disabled,
                None => Exposure::Unknown,
            },
            topology: Topology::Unknown,
            detail: Some("active plugin version and cache path are not recorded in config".into()),
            skills: Vec::new(),
        })
        .collect::<Vec<_>>();
    let mut diagnostics = plugin_diagnostics(policy, Agent::Codex, scope, plugins);
    if !diagnostics.is_empty() {
        diagnostics.push(Diagnostic::new(
            "skills",
            State::Unsupported,
            "external codex user plugin skills inventory origin=plugin ownership=external exposure=unknown topology=unknown policy=unknown activation=unknown detail=the active cache version cannot be selected reliably from config.toml",
        ));
    }
    diagnostics
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
