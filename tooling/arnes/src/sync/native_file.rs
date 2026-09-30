use super::{SyncEntry, SyncState};
use crate::hooks::io::ConfigFile;
use std::path::Path;

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
