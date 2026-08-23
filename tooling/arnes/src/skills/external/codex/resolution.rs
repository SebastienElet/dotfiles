use super::state::{ResolverState, SelectedPlugin};
use crate::files::paths::canonical_within;
use crate::skills::external::manifest;
use crate::skills::external::model::{Exposure, Plugin, Topology};
use std::fs;
use std::path::{Component, Path, PathBuf};

struct PluginIdentity {
    name: String,
    marketplace: String,
}

pub(super) fn resolve(
    home: &Path,
    id: String,
    configured: Option<bool>,
    state: &ResolverState,
) -> Plugin {
    let selected = match selected_plugin(&id, state) {
        Ok(selected) => selected,
        Err(detail) => return unresolved(id, configured, detail),
    };
    if configured.is_some_and(|enabled| enabled != selected.enabled) {
        return unresolved(
            id,
            configured,
            "Codex resolver exposure does not match configuration",
        );
    }
    let identity = match selected_identity(&id, selected) {
        Ok(identity) => identity,
        Err(detail) => return unresolved(id, Some(selected.enabled), detail),
    };
    if let Err(detail) = selected_marketplace(&identity.marketplace, state) {
        return unresolved(id, Some(selected.enabled), detail);
    }
    let artifact = match selected_artifact(selected) {
        Ok(artifact) => artifact,
        Err(detail) => return unresolved(id, Some(selected.enabled), detail),
    };
    inspect_artifact(home, id, identity, artifact, exposure(selected.enabled))
}

fn selected_plugin<'a>(
    id: &str,
    state: &'a ResolverState,
) -> Result<&'a SelectedPlugin, &'static str> {
    let selected = state
        .plugins
        .iter()
        .filter(|plugin| plugin.plugin_id == id)
        .collect::<Vec<_>>();
    match selected.as_slice() {
        [] => Err("Codex resolver did not select this configured plugin"),
        [selected] => Ok(*selected),
        _ => Err("Codex resolver returned duplicate plugin selection"),
    }
}

fn selected_identity(id: &str, selected: &SelectedPlugin) -> Result<PluginIdentity, &'static str> {
    let Some((name, marketplace)) = id.split_once('@') else {
        return Err("Codex resolver plugin identifier is invalid");
    };
    if !safe_segment(name) || !safe_segment(marketplace) || marketplace.contains('@') {
        return Err("Codex resolver plugin identifier is invalid");
    }
    if selected.name != name {
        return Err("Codex resolver plugin name does not match its identifier");
    }
    if selected.marketplace_name != marketplace {
        return Err("Codex resolver marketplace does not match its identifier");
    }
    Ok(PluginIdentity {
        name: name.to_owned(),
        marketplace: marketplace.to_owned(),
    })
}

fn selected_marketplace(marketplace: &str, state: &ResolverState) -> Result<(), &'static str> {
    match state
        .marketplaces
        .iter()
        .filter(|candidate| candidate.name == marketplace)
        .count()
    {
        0 => Err("Codex marketplace selection is missing"),
        1 => Ok(()),
        _ => Err("Codex resolver returned duplicate marketplace selection"),
    }
}

fn selected_artifact(selected: &SelectedPlugin) -> Result<&str, &'static str> {
    if !selected.installed {
        return Err("Codex resolver selected a plugin that is not installed");
    }
    let artifact = selected
        .version
        .as_deref()
        .ok_or("Codex resolver has no active artifact identifier")?;
    if !safe_segment(artifact) {
        return Err("Codex resolver artifact identifier is invalid");
    }
    Ok(artifact)
}

fn inspect_artifact(
    home: &Path,
    id: String,
    identity: PluginIdentity,
    artifact: &str,
    exposure: Exposure,
) -> Plugin {
    let canonical_path = match resolved_artifact_path(home, &identity, artifact) {
        Ok(path) => path,
        Err((path, detail)) => return broken(id, artifact, path, exposure, detail),
    };
    let inspected = manifest::inspect(&canonical_path, &[".codex-plugin/plugin.json"]);
    if inspected.topology == Topology::Healthy
        && inspected.name.as_deref() != Some(identity.name.as_str())
    {
        return broken(
            id,
            artifact,
            canonical_path,
            exposure,
            "plugin manifest name does not match Codex selection",
        );
    }
    Plugin {
        id,
        artifact: Some(artifact.to_owned()),
        version: inspected.version,
        path: Some(canonical_path),
        exposure,
        topology: inspected.topology,
        detail: inspected.detail,
        skills: inspected.skills,
    }
}

fn resolved_artifact_path(
    home: &Path,
    identity: &PluginIdentity,
    artifact: &str,
) -> Result<PathBuf, (PathBuf, &'static str)> {
    let cache = home.join(".codex/plugins/cache");
    let path = cache
        .join(&identity.marketplace)
        .join(&identity.name)
        .join(artifact);
    if fs::symlink_metadata(&path).is_err() {
        return Err((path, "resolved plugin artifact is missing"));
    }
    let Some(canonical_path) = canonical_within(&path, &cache) else {
        return Err((path, "resolved plugin path escapes the Codex plugin cache"));
    };
    let base = cache.join(&identity.marketplace).join(&identity.name);
    let Some(canonical_cache) = fs::canonicalize(&cache).ok() else {
        return Err((path, "Codex plugin cache root could not be resolved"));
    };
    let Some(canonical_base) = canonical_within(&base, &cache) else {
        return Err((path, "resolved plugin path aliases another cache identity"));
    };
    if canonical_base
        != canonical_cache
            .join(&identity.marketplace)
            .join(&identity.name)
        || canonical_path != canonical_base.join(artifact)
    {
        return Err((path, "resolved plugin path aliases another cache identity"));
    }
    Ok(canonical_path)
}

fn safe_segment(value: &str) -> bool {
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

pub(super) fn unresolved(id: String, enabled: Option<bool>, detail: impl Into<String>) -> Plugin {
    Plugin {
        id,
        artifact: None,
        version: None,
        path: None,
        exposure: enabled.map_or(Exposure::Unknown, exposure),
        topology: Topology::Unknown,
        detail: Some(detail.into()),
        skills: Vec::new(),
    }
}

fn broken(
    id: String,
    artifact: &str,
    path: PathBuf,
    exposure: Exposure,
    detail: impl Into<String>,
) -> Plugin {
    Plugin {
        id,
        artifact: Some(artifact.to_owned()),
        version: None,
        path: Some(path),
        exposure,
        topology: Topology::Broken,
        detail: Some(detail.into()),
        skills: Vec::new(),
    }
}

fn exposure(enabled: bool) -> Exposure {
    if enabled {
        Exposure::Enabled
    } else {
        Exposure::Disabled
    }
}
