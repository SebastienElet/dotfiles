use super::SelectedRoot;
use crate::measure::{MeasureError, model::HookAgent};
use serde::Deserialize;
use serde_json::Value as JsonValue;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Read;
use std::path::{Component, Path, PathBuf};

const MAX_MANIFEST_BYTES: u64 = 1_048_576;

pub struct PluginSelection {
    pub roots: Vec<SelectedRoot>,
    pub limitations: Vec<String>,
}

#[derive(Default, Deserialize)]
struct CodexConfig {
    #[serde(default)]
    plugins: BTreeMap<String, CodexPlugin>,
}

#[derive(Deserialize)]
struct CodexPlugin {
    enabled: Option<bool>,
}

pub fn selected(agent: HookAgent, home: &Path) -> Result<PluginSelection, MeasureError> {
    Ok(match agent {
        HookAgent::Codex => codex(home)?,
        HookAgent::ClaudeCode => claude(home)?,
        HookAgent::Cursor => PluginSelection {
            roots: Vec::new(),
            limitations: vec![
                "cursor marketplace and extension activation have no filesystem registry; local plugins fingerprinted"
                    .to_owned(),
            ],
        },
    })
}

fn codex(home: &Path) -> Result<PluginSelection, MeasureError> {
    let mut roots = Vec::new();
    let config = read_manifest(&home.join(".codex/config.toml"))?
        .and_then(|contents| toml::from_str::<CodexConfig>(&contents).ok())
        .unwrap_or_default();
    for (id, plugin) in config.plugins {
        if plugin.enabled == Some(true) {
            roots.extend(codex_plugin_root(home, &id));
        }
    }
    let limitations = if roots.is_empty() {
        Vec::new()
    } else {
        vec![
            "codex active plugin version is not recorded; enabled plugin cache candidates fingerprinted"
                .to_owned(),
        ]
    };
    Ok(PluginSelection { roots, limitations })
}

fn codex_plugin_root(home: &Path, id: &str) -> Option<SelectedRoot> {
    let (plugin, marketplace) = id.split_once('@')?;
    if !safe_name(plugin) || !safe_name(marketplace) {
        return None;
    }
    Some(SelectedRoot {
        label: Path::new("home/.codex/plugins/cache-candidates")
            .join(marketplace)
            .join(plugin),
        path: home
            .join(".codex/plugins/cache")
            .join(marketplace)
            .join(plugin),
    })
}

fn claude(home: &Path) -> Result<PluginSelection, MeasureError> {
    let plugin_root = home.join(".claude/plugins");
    let registry = read_manifest(&plugin_root.join("installed_plugins.json"))?
        .and_then(|contents| serde_json::from_str::<JsonValue>(&contents).ok());
    let mut roots = Vec::new();
    if registry.as_ref().and_then(|value| value.get("version")) == Some(&JsonValue::from(2))
        && let Some(plugins) = registry
            .as_ref()
            .and_then(|value| value.get("plugins"))
            .and_then(JsonValue::as_object)
    {
        for (id, installations) in plugins {
            roots.extend(claude_installations(&plugin_root, id, installations));
        }
    }
    Ok(PluginSelection {
        roots,
        limitations: Vec::new(),
    })
}

fn read_manifest(path: &Path) -> Result<Option<String>, MeasureError> {
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

fn claude_installations(root: &Path, id: &str, value: &JsonValue) -> Vec<SelectedRoot> {
    value
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|installation| installation.get("installPath"))
        .filter_map(JsonValue::as_str)
        .map(PathBuf::from)
        .filter(|path| within(path, root))
        .enumerate()
        .map(|(index, path)| SelectedRoot {
            label: Path::new("home/.claude/plugins/registered")
                .join(id.replace('/', "_"))
                .join(index.to_string()),
            path,
        })
        .collect()
}

fn within(path: &Path, root: &Path) -> bool {
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

fn safe_name(value: &str) -> bool {
    !value.is_empty()
        && value.chars().all(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.')
        })
}
