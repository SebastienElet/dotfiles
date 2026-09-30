use super::{SyncEntry, SyncState};
use crate::Roots;
use crate::hooks::io::ConfigFile;
use crate::manifest::Scope;
use std::fs;
use std::path::Path;

pub(super) mod json;

pub(super) fn scope_root(roots: &Roots, scope: Scope) -> Result<&Path, &'static str> {
    let root = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let metadata = fs::symlink_metadata(root).map_err(|_| "scope root is unavailable")?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("scope root must be a directory without a symbolic link");
    }
    let canonical = root
        .canonicalize()
        .map_err(|_| "scope root cannot be resolved")?;
    if scope == Scope::User {
        let deployed = roots
            .deployment_repository()
            .canonicalize()
            .map_err(|_| "deployment repository cannot be resolved")?;
        if canonical.starts_with(deployed) {
            return Err("user scope aliases a canonical source");
        }
    }
    for repository in [roots.repository(), roots.deployment_repository()] {
        for relative in ["home", "harness", "harness/skills", ".agents/skills"] {
            let source = repository.join(relative);
            match fs::symlink_metadata(&source) {
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => return Err("canonical source boundary is unavailable"),
                Ok(_) => {
                    let source = source
                        .canonicalize()
                        .map_err(|_| "canonical source boundary cannot be resolved")?;
                    if canonical.starts_with(source) {
                        return Err("scope aliases a canonical source");
                    }
                }
            }
        }
    }
    Ok(root)
}

pub(super) fn update_json(
    root: &Path,
    directory: &str,
    name: &str,
    id: &str,
    edit: impl FnOnce(&mut json::Object) -> Result<(), &'static str>,
) -> SyncEntry {
    let Ok(file) = ConfigFile::open(root, directory, name) else {
        return failed(id, "configuration could not be safely opened");
    };
    let original = file.content().map(<[u8]>::to_vec);
    let Ok(mut document) = json::parse(original.as_deref().unwrap_or(b"{}")) else {
        return SyncEntry::new(
            id,
            SyncState::Refused,
            "configuration is malformed or ambiguous",
        );
    };
    let Ok(before) = serde_json::to_vec_pretty(&document) else {
        return failed(id, "configuration could not be rendered");
    };
    if let Err(message) = edit(&mut document) {
        return SyncEntry::new(id, SyncState::Refused, message);
    }
    let Ok(rendered) = serde_json::to_vec_pretty(&document) else {
        return failed(id, "configuration could not be rendered");
    };
    if before == rendered {
        let Some(original) = original else {
            return SyncEntry::new(id, SyncState::Empty, "no managed values require creation");
        };
        return match file.replace(&original) {
            Ok(()) => SyncEntry::new(id, SyncState::Current, "managed values are conforming"),
            Err(_) => failed(id, "configuration changed during synchronization"),
        };
    }
    match file.replace(&rendered) {
        Ok(()) => SyncEntry::new(id, SyncState::Applied, "managed values synchronized"),
        Err(_) => failed(id, "configuration publication failed"),
    }
}

pub(super) fn update_toml(
    root: &Path,
    id: &str,
    edit: impl FnOnce(&mut toml::Table) -> Result<(), &'static str>,
) -> SyncEntry {
    let Ok(file) = ConfigFile::open(root, ".codex", "config.toml") else {
        return failed(id, "configuration could not be safely opened");
    };
    let original = file.content().unwrap_or_default().to_vec();
    let Some(mut document) = std::str::from_utf8(&original)
        .ok()
        .and_then(|text| toml::from_str::<toml::Table>(text).ok())
    else {
        return SyncEntry::new(id, SyncState::Refused, "configuration is malformed");
    };
    let Ok(before) = toml::to_string(&document) else {
        return failed(id, "configuration could not be rendered");
    };
    if let Err(message) = edit(&mut document) {
        return SyncEntry::new(id, SyncState::Refused, message);
    }
    let Ok(rendered) = toml::to_string(&document) else {
        return failed(id, "configuration could not be rendered");
    };
    if before == rendered {
        return match file.replace(&original) {
            Ok(()) => SyncEntry::new(id, SyncState::Current, "managed values are conforming"),
            Err(_) => failed(id, "configuration changed during synchronization"),
        };
    }
    match file.replace(rendered.as_bytes()) {
        Ok(()) => SyncEntry::new(id, SyncState::Applied, "managed values synchronized"),
        Err(_) => failed(id, "configuration publication failed"),
    }
}

fn failed(id: &str, message: &str) -> SyncEntry {
    SyncEntry::new(id, SyncState::Failed, message)
}
