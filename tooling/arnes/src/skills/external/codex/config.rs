use super::Exposure;
use crate::files::paths::{ancestor_within, canonical_within};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Default, Deserialize)]
pub(super) struct Config {
    #[serde(default)]
    pub plugins: BTreeMap<String, PluginConfig>,
    #[serde(default)]
    skills: SkillsConfig,
}

#[derive(Deserialize)]
pub(super) struct PluginConfig {
    pub enabled: Option<bool>,
}

#[derive(Default, Deserialize)]
struct SkillsConfig {
    #[serde(default)]
    config: Vec<SkillConfig>,
}

#[derive(Deserialize)]
struct SkillConfig {
    path: Option<PathBuf>,
    name: Option<String>,
    enabled: bool,
}

impl Config {
    pub(super) fn skill_exposure(&self, skill_file: &Path) -> Exposure {
        let name = self
            .skills
            .config
            .iter()
            .any(|entry| entry.name.is_some())
            .then(|| skill_name(skill_file))
            .flatten();
        self.skills
            .config
            .iter()
            .rev()
            .find_map(|entry| entry.matching_exposure(skill_file, name.as_deref()))
            .unwrap_or(Exposure::Enabled)
    }
}

impl SkillConfig {
    fn matching_exposure(&self, skill_file: &Path, skill_name: Option<&str>) -> Option<Exposure> {
        let matches = match (self.path.as_deref(), self.name.as_deref()) {
            (Some(path), None) => same_path(path, skill_file),
            (None, Some(name)) if !name.trim().is_empty() => {
                let Some(skill_name) = skill_name else {
                    return Some(Exposure::Unknown);
                };
                name.trim() == skill_name
            }
            _ => false,
        };
        matches.then_some(if self.enabled {
            Exposure::Enabled
        } else {
            Exposure::Disabled
        })
    }
}

pub(super) fn load(path: &Path, root: &Path) -> Result<Config, &'static str> {
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
    let value: toml::Value = toml::from_str(&contents).map_err(|_| "config is invalid TOML")?;
    value
        .try_into()
        .map_err(|_| "config has invalid plugin or skill settings")
}

fn skill_name(skill_file: &Path) -> Option<String> {
    #[derive(Deserialize)]
    struct Metadata {
        name: Option<String>,
    }

    let root = skill_file.parent()?.parent()?;
    let path = canonical_within(skill_file, root)?;
    if !fs::metadata(&path).ok()?.is_file() {
        return None;
    }
    let contents = fs::read_to_string(&path).ok()?;
    let mut lines = contents.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let mut frontmatter = Vec::new();
    for line in lines {
        if line.trim() == "---" {
            let metadata: Metadata = serde_yaml_ng::from_str(&frontmatter.join("\n")).ok()?;
            let name = metadata.name.unwrap_or_default();
            let name = if name.trim().is_empty() {
                path.parent()?.file_name()?.to_str().unwrap_or("skill")
            } else {
                &name
            };
            let name = name.split_whitespace().collect::<Vec<_>>().join(" ");
            return Some(if name.is_empty() {
                "skill".to_owned()
            } else {
                name
            });
        }
        frontmatter.push(line);
    }
    None
}

fn same_path(left: &Path, right: &Path) -> bool {
    left == right
        || fs::canonicalize(left)
            .ok()
            .zip(fs::canonicalize(right).ok())
            .is_some_and(|(left, right)| left == right)
}
