use super::{PluginSelection, read_manifest, safe_label, within};
use crate::measure::MeasureError;
use crate::measure::fingerprint::SelectedRoot;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

const MAX_VERSIONS: usize = 512;

#[derive(Default, Deserialize)]
struct Config {
    #[serde(default)]
    plugins: BTreeMap<String, Plugin>,
}

#[derive(Deserialize)]
struct Plugin {
    enabled: Option<bool>,
}

pub fn selected(home: &Path) -> Result<PluginSelection, MeasureError> {
    let config = read_manifest(&home.join(".codex/config.toml"))?
        .map(|value| toml::from_str::<Config>(&value))
        .transpose()
        .map_err(|error| MeasureError::new(error.to_string()))?
        .unwrap_or_default();
    let mut selection = PluginSelection {
        roots: Vec::new(),
        limitations: Vec::new(),
        markers: Vec::new(),
    };
    for (id, plugin) in config.plugins {
        if plugin.enabled == Some(true) {
            select_enabled(home, &id, &mut selection)?;
        }
    }
    Ok(selection)
}

fn select_enabled(
    home: &Path,
    id: &str,
    selection: &mut PluginSelection,
) -> Result<(), MeasureError> {
    let Some((name, marketplace)) = id.rsplit_once('@') else {
        invalid_id(id, selection);
        return Ok(());
    };
    if !normal_component(name) || !normal_component(marketplace) {
        invalid_id(id, selection);
        return Ok(());
    }
    let cache = home.join(".codex/plugins/cache");
    let base = cache.join(marketplace).join(name);
    if !within(&cache, home) || !within(&base, &cache) {
        selection.markers.push(format!("codex:{id}:cache-escape"));
        add_limitation(selection, "codex enabled plugin cache root is unresolved");
        return Ok(());
    }
    let versions = versions(&base)?;
    match versions.as_slice() {
        [(version, path)] => selection.roots.push(SelectedRoot {
            label: Path::new("home/.codex/plugins/active")
                .join(safe_label(id))
                .join(safe_label(version)),
            path: path.to_owned(),
            bounded: true,
        }),
        [] => {
            selection.markers.push(format!("codex:{id}:missing"));
            add_limitation(selection, "codex enabled plugin cache version is missing");
        }
        _ => {
            let names = versions
                .iter()
                .fold(String::new(), |mut names, (version, _)| {
                    names.push_str(&format!("{}:{version}", version.len()));
                    names
                });
            selection
                .markers
                .push(format!("codex:{id}:ambiguous:{names}"));
            add_limitation(selection, "codex enabled plugin cache version is ambiguous");
        }
    }
    Ok(())
}

fn invalid_id(id: &str, selection: &mut PluginSelection) {
    selection.markers.push(format!("codex:{id}:invalid-id"));
    add_limitation(selection, "codex enabled plugin identifier is unresolved");
}

fn normal_component(value: &str) -> bool {
    let path = Path::new(value);
    let mut components = path.components();
    matches!(components.next(), Some(Component::Normal(component)) if path.as_os_str() == component)
        && components.next().is_none()
}

fn versions(base: &Path) -> Result<Vec<(String, PathBuf)>, MeasureError> {
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut versions = entries
        .take(MAX_VERSIONS + 1)
        .collect::<Result<Vec<_>, _>>()?;
    if versions.len() > MAX_VERSIONS {
        return Err(MeasureError::new("codex plugin cache exceeds 512 versions"));
    }
    versions.retain(|entry| entry.file_type().is_ok_and(|kind| kind.is_dir()));
    versions.sort_by_key(|entry| entry.file_name());
    Ok(versions
        .into_iter()
        .map(|entry| {
            (
                entry.file_name().to_string_lossy().into_owned(),
                entry.path(),
            )
        })
        .collect())
}

fn add_limitation(selection: &mut PluginSelection, limitation: &str) {
    if !selection
        .limitations
        .iter()
        .any(|value| value == limitation)
    {
        selection.limitations.push(limitation.to_owned());
    }
}
