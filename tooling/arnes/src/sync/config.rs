use super::native_file::{self, json};
use super::{SyncEntry, SyncState};
use crate::Roots;
use crate::manifest::{Agent, Manifest, Scope, UserConfig};

pub(super) fn synchronize(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Vec<SyncEntry> {
    if scope != Scope::User {
        return vec![SyncEntry::new(
            "config",
            SyncState::Unsupported,
            "user defaults are never applied to project configuration",
        )];
    }
    let Some(config) = manifest.user_config(agent) else {
        return vec![SyncEntry::new(
            "config",
            SyncState::Empty,
            "no managed user defaults selected",
        )];
    };
    let root = match native_file::scope_root(roots, scope) {
        Ok(root) => root,
        Err(message) => return vec![SyncEntry::new("config", SyncState::Refused, message)],
    };
    let path = match agent {
        Agent::Claude => ".claude/settings.json",
        Agent::Cursor => ".cursor/cli-config.json",
        Agent::Codex => ".codex/config.toml",
    };
    let protected = super::sources::publication_paths(&[path.into()])
        .and_then(|paths| super::sources::protect_mutations(roots, manifest, scope, &paths));
    if let Err(message) = protected {
        return vec![SyncEntry::new("config", SyncState::Refused, message)];
    }
    let entry = match agent {
        Agent::Claude => {
            native_file::update_json(root, ".claude", "settings.json", "config", |document| {
                json::set(document, "model", &config.model)?;
                if let Some(effort) = &config.effort {
                    json::set(document, "effortLevel", effort)?;
                }
                if let Some(window) = config.auto_compact_window {
                    json::set(document, "autoCompactWindow", &window)?;
                }
                Ok(())
            })
        }
        Agent::Cursor => {
            native_file::update_json(root, ".cursor", "cli-config.json", "config", |document| {
                let mut model = json::child(document, "model")?;
                if json::set(&mut model, "modelId", &config.model)? {
                    json::put(document, "model", &model)?;
                }
                if let Some(max_mode) = config.max_mode {
                    json::set(document, "maxMode", &max_mode)?;
                }
                Ok(())
            })
        }
        Agent::Codex => {
            native_file::update_toml(root, "config", |document| update_codex(document, config))
        }
    };
    vec![entry]
}

fn update_codex(document: &mut toml::Table, config: &UserConfig) -> Result<(), &'static str> {
    set_toml(document, "model", toml::Value::String(config.model.clone()))?;
    if let Some(effort) = &config.effort {
        set_toml(
            document,
            "model_reasoning_effort",
            toml::Value::String(effort.clone()),
        )?;
    }
    for (key, value) in [
        ("model_context_window", config.context_window),
        ("model_auto_compact_token_limit", config.auto_compact_window),
    ] {
        if let Some(value) = value {
            let value = i64::try_from(value)
                .map_err(|_| "managed integer exceeds the native TOML range")?;
            set_toml(document, key, toml::Value::Integer(value))?;
        }
    }
    Ok(())
}

fn set_toml(
    document: &mut toml::Table,
    key: &str,
    expected: toml::Value,
) -> Result<(), &'static str> {
    if let Some(actual) = document.get(key) {
        let same_type = matches!(
            (actual, &expected),
            (toml::Value::String(_), toml::Value::String(_))
                | (toml::Value::Integer(_), toml::Value::Integer(_))
        );
        if !same_type {
            return Err("managed value has an unsupported type");
        }
    }
    document.insert(key.to_owned(), expected);
    Ok(())
}
