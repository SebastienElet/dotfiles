mod codex;
mod cursor;
use super::SelectedRoot;
use crate::measure::{MeasureError, model::HookAgent};
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const MAX_MANIFEST_BYTES: u64 = 1_048_576;

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

fn unavailable_runtime(limitation: &str) -> PluginSelection {
    PluginSelection {
        roots: Vec::new(),
        limitations: vec![limitation.to_owned()],
        markers: Vec::new(),
    }
}

fn claude(home: &Path, repository: &Path) -> Result<PluginSelection, MeasureError> {
    let plugin_root = home.join(".claude/plugins");
    let registry = read_json(&plugin_root.join("installed_plugins.json"))?;
    let enabled: Vec<String> = claude_settings(home, repository)?
        .into_iter()
        .filter_map(|(id, enabled)| enabled.then_some(id))
        .collect();
    if enabled.is_empty() {
        return Ok(PluginSelection {
            roots: Vec::new(),
            limitations: Vec::new(),
            markers: Vec::new(),
        });
    }
    let Some(plugins) = registry
        .as_ref()
        .filter(|value| value.get("version") == Some(&JsonValue::from(2)))
        .and_then(|value| value.get("plugins"))
        .and_then(JsonValue::as_object)
    else {
        return Ok(unavailable_runtime(
            "claude plugin registry is not observable in supported version 2; enabled plugin files excluded",
        ));
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
    Ok(PluginSelection {
        roots,
        limitations: unresolved
            .then(|| {
                "claude enabled plugin installation is not uniquely observable; unresolved plugin files excluded"
                    .to_owned()
            })
            .into_iter()
            .collect(),
        markers: Vec::new(),
    })
}

fn claude_settings(home: &Path, repository: &Path) -> Result<BTreeMap<String, bool>, MeasureError> {
    let mut enabled = BTreeMap::new();
    for path in [
        home.join(".claude/settings.json"),
        repository.join(".claude/settings.json"),
        repository.join(".claude/settings.local.json"),
    ] {
        let Some(settings) = read_json(&path)? else {
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

fn read_json(path: &Path) -> Result<Option<JsonValue>, MeasureError> {
    read_manifest(path)?
        .map(|contents| serde_json::from_str(&contents).map_err(MeasureError::from))
        .transpose()
}

pub(super) fn read_manifest(path: &Path) -> Result<Option<String>, MeasureError> {
    let mut file = match File::open(path) {
        Ok(file) => file,
        Err(_) => return Ok(None),
    };
    let mut bytes = Vec::new();
    file.by_ref()
        .take(MAX_MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_MANIFEST_BYTES {
        return Err(MeasureError::new("plugin manifest exceeds 1048576 bytes"));
    }
    Ok(String::from_utf8(bytes).ok())
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
