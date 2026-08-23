use super::super::model::Exposure;
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within};
use crate::manifest::Scope;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

pub(super) fn load(
    roots: &Roots,
    scope: Scope,
) -> (Result<BTreeMap<String, bool>, ()>, Vec<Diagnostic>) {
    let mut merged = BTreeMap::new();
    let paths = match scope {
        Scope::User => vec![(roots.home().join(".claude/settings.json"), roots.home())],
        Scope::Project => vec![
            (roots.home().join(".claude/settings.json"), roots.home()),
            (
                roots.repository().join(".claude/settings.json"),
                roots.repository(),
            ),
            (
                roots.repository().join(".claude/settings.local.json"),
                roots.repository(),
            ),
        ],
    };
    for (path, base) in paths {
        match load_one(&path, base) {
            Ok(values) => merged.extend(values),
            Err(detail) => {
                return (
                    Err(()),
                    vec![Diagnostic::new(
                        "skills",
                        State::Error,
                        format!(
                            "external claude {scope} plugin settings origin=plugin ownership=external exposure=unknown topology=unreadable policy=unknown activation=unknown path={} detail={detail}",
                            path.display(),
                        ),
                    )],
                );
            }
        }
    }
    (Ok(merged), Vec::new())
}

pub(super) fn exposure(settings: &Result<BTreeMap<String, bool>, ()>, id: &str) -> Exposure {
    match settings {
        Ok(settings) => match settings.get(id) {
            Some(true) => Exposure::Enabled,
            Some(false) => Exposure::Disabled,
            None => Exposure::Unknown,
        },
        Err(()) => Exposure::Unknown,
    }
}

fn load_one(path: &Path, base: &Path) -> Result<BTreeMap<String, bool>, &'static str> {
    match fs::symlink_metadata(path) {
        Ok(_) if canonical_within(path, base).is_none() => {
            return Err("settings resolve outside their scope");
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return if ancestor_within(path, base) {
                Ok(BTreeMap::new())
            } else {
                Err("settings parent resolves outside its scope")
            };
        }
        Err(_) => return Err("settings metadata could not be read"),
    }
    let contents = fs::read_to_string(path).map_err(|_| "settings could not be read")?;
    let value: Value = serde_json::from_str(&contents).map_err(|_| "settings are invalid JSON")?;
    let Some(enabled) = value.get("enabledPlugins") else {
        return Ok(BTreeMap::new());
    };
    serde_json::from_value(enabled.clone()).map_err(|_| "enabledPlugins must map ids to booleans")
}
