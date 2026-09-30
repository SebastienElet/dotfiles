use super::native_file::update_toml;
use super::{SyncEntry, SyncState};
use crate::Roots;
use crate::manifest::{Agent, Manifest, Scope};

pub(super) fn synchronize(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Vec<SyncEntry> {
    if agent != Agent::Codex {
        return vec![SyncEntry::new(
            "statusline",
            SyncState::Unsupported,
            "statusline synchronization supports Codex only",
        )];
    }
    let Some(declaration) = manifest
        .statuslines()
        .find(|declaration| declaration.agent == agent && declaration.scope == scope)
    else {
        return vec![SyncEntry::new(
            "statusline",
            SyncState::Empty,
            "no managed statusline selected",
        )];
    };
    let root = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    if scope == Scope::User && aliases_source(roots) {
        return vec![SyncEntry::new(
            "statusline",
            SyncState::Refused,
            "user scope aliases a canonical source",
        )];
    }
    let protection = super::sources::publication_paths(&[".codex/config.toml".into()])
        .and_then(|paths| super::sources::protect_mutations(roots, manifest, scope, &paths));
    if let Err(message) = protection {
        return vec![SyncEntry::new("statusline", SyncState::Refused, message)];
    }
    let items = declaration
        .items
        .iter()
        .cloned()
        .map(toml::Value::String)
        .collect();
    vec![update_toml(root, "statusline", |document| {
        let tui = document
            .entry("tui")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()
            .ok_or("tui must be a table")?;
        if let Some(value) = tui.get("status_line")
            && !value
                .as_array()
                .is_some_and(|items| items.iter().all(toml::Value::is_str))
        {
            return Err("tui.status_line must be an array of strings");
        }
        tui.insert("status_line".to_owned(), toml::Value::Array(items));
        Ok(())
    })]
}

fn aliases_source(roots: &Roots) -> bool {
    let Ok(home) = roots.home().canonicalize() else {
        return false;
    };
    let mut sources = vec![roots.deployment_repository().to_owned()];
    for repository in [roots.repository(), roots.deployment_repository()] {
        sources.extend(
            ["home", "harness", "harness/skills", ".agents/skills"]
                .map(|directory| repository.join(directory)),
        );
    }
    sources
        .into_iter()
        .filter_map(|source| source.canonicalize().ok())
        .any(|source| home.starts_with(source))
}
