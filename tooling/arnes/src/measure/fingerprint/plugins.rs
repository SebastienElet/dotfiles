mod codex;
mod cursor;
mod manifest;
use super::SelectedRoot;
use crate::measure::{MeasureError, model::HookAgent};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub struct PluginSelection {
    pub roots: Vec<SelectedRoot>,
    pub limitations: Vec<String>,
    pub markers: Vec<String>,
}

pub fn selected(
    agent: HookAgent,
    home: &Path,
    repository: &Path,
) -> Result<PluginSelection, MeasureError> {
    Ok(match agent {
        HookAgent::Codex => codex::selected(home)?,
        HookAgent::ClaudeCode => claude(home, repository)?,
        HookAgent::Cursor => cursor::selected(home)?,
    })
}

fn claude(home: &Path, repository: &Path) -> Result<PluginSelection, MeasureError> {
    let plugin_root = home.join(".claude/plugins");
    let mut selection = PluginSelection {
        roots: Vec::new(),
        limitations: Vec::new(),
        markers: Vec::new(),
    };
    let (registry, marker) = read_json(&plugin_root.join("installed_plugins.json"))?;
    add_manifest_marker(&mut selection, "claude:registry", marker);
    let enabled: Vec<String> = claude_settings(home, repository, &mut selection)?
        .into_iter()
        .filter_map(|(id, enabled)| enabled.then_some(id))
        .collect();
    if enabled.is_empty() {
        return Ok(selection);
    }
    let Some(plugins) = registry
        .as_ref()
        .filter(|value| value.get("version") == Some(&JsonValue::from(2)))
        .and_then(|value| value.get("plugins"))
        .and_then(JsonValue::as_object)
    else {
        selection.limitations.push(
            "claude plugin registry is not observable in supported version 2; enabled plugin files excluded"
                .to_owned(),
        );
        return Ok(selection);
    };
    let mut roots = Vec::new();
    let mut unresolved = false;
    for id in enabled {
        match plugins
            .get(&id)
            .and_then(|value| claude_installation(&plugin_root, &id, value))
        {
            Some(root) => roots.push(root),
            None => unresolved = true,
        }
    }
    selection.roots = roots;
    selection.limitations.extend(unresolved.then(|| {
        "claude enabled plugin installation is not uniquely observable; unresolved plugin files excluded"
            .to_owned()
    }));
    Ok(selection)
}

fn claude_settings(
    home: &Path,
    repository: &Path,
    selection: &mut PluginSelection,
) -> Result<BTreeMap<String, bool>, MeasureError> {
    let mut enabled = BTreeMap::new();
    for (label, path) in [
        ("claude:settings:user", home.join(".claude/settings.json")),
        (
            "claude:settings:project",
            repository.join(".claude/settings.json"),
        ),
        (
            "claude:settings:project-local",
            repository.join(".claude/settings.local.json"),
        ),
    ] {
        let (settings, marker) = read_json(&path)?;
        add_manifest_marker(selection, label, marker);
        let Some(settings) = settings else {
            continue;
        };
        if let Some(values) = settings
            .get("enabledPlugins")
            .and_then(JsonValue::as_object)
        {
            for (id, value) in values {
                if let Some(value) = value.as_bool() {
                    enabled.insert(id.to_owned(), value);
                }
            }
        }
    }
    Ok(enabled)
}

fn read_json(path: &Path) -> Result<(Option<JsonValue>, Option<String>), MeasureError> {
    let manifest = manifest::read(path)?;
    let value = manifest
        .contents
        .map(|contents| serde_json::from_str(&contents).map_err(MeasureError::from))
        .transpose()?;
    Ok((value, manifest.marker))
}

fn add_manifest_marker(selection: &mut PluginSelection, label: &str, marker: Option<String>) {
    if let Some(marker) = marker {
        selection.markers.push(format!("{label}:{marker}"));
        selection
            .limitations
            .push("oversized plugin manifest uses size and boundary windows".to_owned());
    }
}

fn claude_installation(root: &Path, id: &str, value: &JsonValue) -> Option<SelectedRoot> {
    let installations = value.as_array()?;
    if installations.len() != 1 {
        return None;
    }
    let installation = installations.first()?;
    let path = PathBuf::from(installation.get("installPath")?.as_str()?);
    if !within(&path, root) {
        return None;
    }
    let version = installation
        .get("version")
        .and_then(JsonValue::as_str)
        .unwrap_or("unknown");
    Some(SelectedRoot {
        label: Path::new("home/.claude/plugins/active")
            .join(safe_label(id))
            .join(safe_label(version)),
        path,
        bounded: true,
    })
}

pub(super) fn within(path: &Path, root: &Path) -> bool {
    if !path.is_absolute()
        || !path.starts_with(root)
        || path
            .components()
            .any(|component| matches!(component, Component::ParentDir))
    {
        return false;
    }
    match (fs::canonicalize(path), fs::canonicalize(root)) {
        (Ok(path), Ok(root)) => path.starts_with(root),
        (Err(_), _) => true,
        _ => false,
    }
}

pub(super) fn safe_label(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '@') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
