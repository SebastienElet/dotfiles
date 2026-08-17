use super::{installation_scope, registry_diagnostic, settings};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within};
use crate::manifest::Scope;
use crate::skills::external::manifest;
use crate::skills::external::model::{Exposure, Plugin, Topology};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
struct Registry {
    version: u64,
    plugins: BTreeMap<String, Vec<Installation>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Installation {
    scope: String,
    install_path: PathBuf,
    version: String,
}

pub(super) fn diagnose(
    roots: &Roots,
    scope: Scope,
    settings: &Result<BTreeMap<String, bool>, ()>,
) -> (Vec<Plugin>, Vec<Diagnostic>) {
    let plugin_root = roots.home().join(".claude/plugins");
    let registry_path = plugin_root.join("installed_plugins.json");
    if !plugin_root_supported(&plugin_root, roots.home()) {
        return (
            Vec::new(),
            vec![registry_diagnostic(
                scope,
                "plugin root",
                &plugin_root,
                State::Error,
                "broken",
                "plugin root is unreadable or resolves outside HOME",
            )],
        );
    }
    match load(&registry_path, &plugin_root) {
        Ok(Some(registry)) if registry.version == 2 => (
            installed_plugins(&plugin_root, scope, settings, registry),
            Vec::new(),
        ),
        Ok(Some(registry)) => (
            Vec::new(),
            vec![registry_diagnostic(
                scope,
                "plugin registry",
                &registry_path,
                State::Unsupported,
                "unknown",
                &format!("unsupported registry version {}", registry.version),
            )],
        ),
        Ok(None) => (Vec::new(), Vec::new()),
        Err(detail) => (
            Vec::new(),
            vec![registry_diagnostic(
                scope,
                "plugin registry",
                &registry_path,
                State::Error,
                "unreadable",
                detail,
            )],
        ),
    }
}

fn load(path: &Path, root: &Path) -> Result<Option<Registry>, &'static str> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("registry metadata could not be read"),
    }
    if canonical_within(path, root).is_none() {
        return Err("registry resolves outside the plugin root");
    }
    let contents = fs::read_to_string(path).map_err(|_| "registry could not be read")?;
    serde_json::from_str(&contents)
        .map(Some)
        .map_err(|_| "registry is invalid JSON")
}

fn plugin_root_supported(root: &Path, home: &Path) -> bool {
    match fs::symlink_metadata(root) {
        Ok(_) => {
            canonical_within(root, home).is_some()
                && fs::metadata(root).is_ok_and(|metadata| metadata.is_dir())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => ancestor_within(root, home),
        Err(_) => false,
    }
}

fn installed_plugins(
    plugin_root: &Path,
    scope: Scope,
    settings: &Result<BTreeMap<String, bool>, ()>,
    registry: Registry,
) -> Vec<Plugin> {
    registry
        .plugins
        .into_iter()
        .flat_map(|(id, installations)| {
            plugins_for_id(plugin_root, scope, settings, id, installations)
        })
        .collect()
}

fn plugins_for_id(
    plugin_root: &Path,
    scope: Scope,
    settings: &Result<BTreeMap<String, bool>, ()>,
    id: String,
    installations: Vec<Installation>,
) -> Vec<Plugin> {
    let (known, unknown): (Vec<_>, Vec<_>) = installations
        .into_iter()
        .partition(|installation| installation_scope(&installation.scope).is_some());
    let exposure = settings::exposure(settings, &id);
    let mut plugins = Vec::new();
    if !unknown.is_empty() {
        let scopes = unknown
            .iter()
            .map(|installation| installation.scope.as_str())
            .collect::<Vec<_>>()
            .join(",");
        plugins.push(Plugin {
            id: id.clone(),
            version: None,
            path: None,
            exposure: Exposure::Unknown,
            topology: Topology::Unknown,
            detail: Some(format!("unsupported installation scopes {scopes}")),
            skills: Vec::new(),
        });
    }
    let matching = known
        .into_iter()
        .filter(|installation| installation_scope(&installation.scope) == Some(scope))
        .collect::<Vec<_>>();
    if let Some(plugin) = matching_plugin(plugin_root, id, matching, exposure) {
        plugins.push(plugin);
    }
    plugins
}

fn matching_plugin(
    plugin_root: &Path,
    id: String,
    matching: Vec<Installation>,
    exposure: Exposure,
) -> Option<Plugin> {
    if matching.is_empty() {
        return None;
    }
    if matching.len() == 1 {
        return Some(inspect_installation(
            plugin_root,
            id,
            matching.into_iter().next().unwrap(),
            exposure,
        ));
    }
    let versions = matching
        .iter()
        .map(|installation| installation.version.as_str())
        .collect::<Vec<_>>()
        .join(",");
    Some(Plugin {
        id,
        version: None,
        path: None,
        exposure,
        topology: Topology::Broken,
        detail: Some(format!("ambiguous installed versions {versions}")),
        skills: Vec::new(),
    })
}

fn inspect_installation(
    plugin_root: &Path,
    id: String,
    installation: Installation,
    exposure: Exposure,
) -> Plugin {
    let path = installation.install_path;
    let invalid = if !ancestor_within(&path, plugin_root) {
        Some("installed plugin path escapes the plugin root")
    } else if fs::metadata(&path).is_err() {
        Some("installed plugin path is missing")
    } else if canonical_within(&path, plugin_root).is_none() {
        Some("installed plugin path escapes the plugin root")
    } else {
        None
    };
    if let Some(detail) = invalid {
        return Plugin {
            id,
            version: Some(installation.version),
            path: Some(path),
            exposure,
            topology: Topology::Broken,
            detail: Some(detail.to_owned()),
            skills: Vec::new(),
        };
    }
    let inspected = manifest::inspect(&path, &[".claude-plugin/plugin.json"]);
    let version = installation.version;
    let mismatch = inspected
        .version
        .as_deref()
        .is_some_and(|manifest_version| manifest_version != version.as_str());
    Plugin {
        id,
        version: Some(version),
        path: Some(path),
        exposure,
        topology: if mismatch {
            Topology::Broken
        } else {
            inspected.topology
        },
        detail: if mismatch {
            Some("registry and plugin manifest versions differ".to_owned())
        } else {
            inspected.detail
        },
        skills: inspected.skills,
    }
}
