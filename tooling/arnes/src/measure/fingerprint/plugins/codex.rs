use super::{PluginSelection, manifest, safe_label, within};
use crate::measure::MeasureError;
use crate::measure::fingerprint::SelectedRoot;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BinaryHeap};
use std::fmt::Write;
use std::fs;
use std::os::unix::ffi::OsStrExt;
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
    let manifest = manifest::read(&home.join(".codex/config.toml"))?;
    let config = manifest
        .contents
        .map(|value| toml::from_str::<Config>(&value))
        .transpose()
        .map_err(|error| MeasureError::new(error.to_string()))?
        .unwrap_or_default();
    let mut selection = PluginSelection {
        roots: Vec::new(),
        limitations: Vec::new(),
        markers: Vec::new(),
    };
    if let Some(marker) = manifest.marker {
        selection.markers.push(format!("codex:config:{marker}"));
        add_limitation(
            &mut selection,
            "oversized plugin manifest uses size and boundary windows",
        );
    }
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
    match versions.entries.as_slice() {
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
            selection.markers.push(format!(
                "codex:{id}:ambiguous:{}:{}",
                versions.total,
                hex(&versions.aggregate)
            ));
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

struct Versions {
    entries: Vec<(String, PathBuf)>,
    total: usize,
    aggregate: [u8; 32],
}

fn versions(base: &Path) -> Result<Versions, MeasureError> {
    let entries = match fs::read_dir(base) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(empty_versions()),
        Err(error) => return Err(error.into()),
    };
    let mut selected = BinaryHeap::new();
    let mut aggregate = [0_u8; 32];
    let mut total = 0;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name();
        for (target, byte) in aggregate.iter_mut().zip(Sha256::digest(name.as_bytes())) {
            *target ^= byte;
        }
        selected.push((name, entry.path()));
        total += 1;
        if selected.len() > MAX_VERSIONS {
            selected.pop();
        }
    }
    let mut entries = selected.into_vec();
    entries.sort();
    Ok(Versions {
        entries: entries
            .into_iter()
            .map(|(name, path)| (name.to_string_lossy().into_owned(), path))
            .collect(),
        total,
        aggregate,
    })
}

fn empty_versions() -> Versions {
    Versions {
        entries: Vec::new(),
        total: 0,
        aggregate: [0; 32],
    }
}

fn hex(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(value, "{byte:02x}").expect("writing to a string cannot fail");
    }
    value
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
