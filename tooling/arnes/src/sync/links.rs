use super::selection::LinkIntent;
use super::{SyncEntry, SyncState};
use rustix::fs::{AtFlags, Mode, OFlags};
use std::fs::{self, File};
use std::io;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

#[cfg(test)]
mod tests;

pub(super) struct PreparedLink {
    intent: LinkIntent,
    source: PathBuf,
    protected_source: PathBuf,
    identity: (u64, u64),
    current: bool,
}

pub(super) fn synchronize(selected: Vec<LinkIntent>) -> Vec<SyncEntry> {
    let mut prepared = Vec::new();
    let mut failures = Vec::new();
    for intent in selected {
        match prepare(intent) {
            Ok(link) => prepared.push(link),
            Err(entry) => failures.push(entry),
        }
    }
    if failures.is_empty()
        && let Some(entry) = aliases(&prepared)
    {
        failures.push(entry);
    }
    if !failures.is_empty() {
        failures.extend(prepared.into_iter().map(|link| {
            SyncEntry::new(
                link.intent.id,
                SyncState::Refused,
                "selection failed validation; projection preserved",
            )
        }));
        return failures;
    }
    prepared.into_iter().map(publish).collect()
}

pub(super) fn prepare(intent: LinkIntent) -> Result<PreparedLink, SyncEntry> {
    let refused = |message| SyncEntry::new(&intent.id, SyncState::Refused, message);
    let source = intent.source().map_err(refused)?;
    let protected_source = intent.protected_source().map_err(refused)?;
    let metadata =
        fs::metadata(&source).map_err(|_| refused("source could not be read".to_owned()))?;
    let identity = (metadata.dev(), metadata.ino());
    validate_parent(&intent.root, &intent.destination).map_err(refused)?;
    let destination = intent.root.join(&intent.destination);
    let current = match fs::symlink_metadata(&destination) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => false,
        Ok(metadata) if metadata.file_type().is_symlink() => {
            if fs::canonicalize(&destination).ok().as_ref() != Some(&source) {
                return Err(refused(
                    "destination is a divergent or dangling link".to_owned(),
                ));
            }
            true
        }
        Ok(_) => {
            return Err(refused(
                "destination exists and is not the expected symbolic link".to_owned(),
            ));
        }
        Err(_) => return Err(refused("destination could not be inspected".to_owned())),
    };
    Ok(PreparedLink {
        intent,
        source,
        protected_source,
        identity,
        current,
    })
}

fn aliases(prepared: &[PreparedLink]) -> Option<SyncEntry> {
    for link in prepared {
        let root = fs::canonicalize(&link.intent.root).ok()?;
        let destination = root.join(&link.intent.destination);
        for other in prepared {
            if destination.starts_with(&other.source)
                || destination.starts_with(&other.protected_source)
            {
                return Some(SyncEntry::new(
                    &link.intent.id,
                    SyncState::Refused,
                    "destination aliases a canonical source",
                ));
            }
        }
    }
    None
}

fn validate_parent(root: &Path, destination: &Path) -> Result<(), String> {
    let metadata =
        fs::symlink_metadata(root).map_err(|_| "scope root could not be inspected".to_owned())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("scope root must be a directory, not a symbolic link".to_owned());
    }
    let parent = destination
        .parent()
        .ok_or_else(|| "destination has no parent".to_owned())?;
    let mut path = root.to_owned();
    for component in parent.components() {
        let Component::Normal(component) = component else {
            return Err("destination must stay within its scope".to_owned());
        };
        path.push(component);
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Ok(_) => {
                return Err(
                    "destination parent must be a directory, not a symbolic link".to_owned(),
                );
            }
            Err(_) => return Err("destination parent could not be inspected".to_owned()),
        }
    }
    Ok(())
}

pub(super) fn publish(link: PreparedLink) -> SyncEntry {
    match publish_link(&link) {
        Ok(state) => SyncEntry::new(link.intent.id, state, "managed link is conforming"),
        Err(message) => SyncEntry::new(link.intent.id, SyncState::Failed, message),
    }
}

fn publish_link(link: &PreparedLink) -> Result<SyncState, String> {
    let current_source = link.intent.source()?;
    let metadata =
        fs::metadata(&current_source).map_err(|_| "source could not be read".to_owned())?;
    if current_source != link.source || (metadata.dev(), metadata.ino()) != link.identity {
        return Err("source changed during synchronization".to_owned());
    }
    validate_parent(&link.intent.root, &link.intent.destination)?;
    if link.current {
        if fs::symlink_metadata(link.intent.root.join(&link.intent.destination))
            .is_ok_and(|metadata| metadata.file_type().is_symlink())
            && fs::canonicalize(link.intent.root.join(&link.intent.destination))
                .ok()
                .as_ref()
                == Some(&link.source)
        {
            return Ok(SyncState::Current);
        }
        return Err("destination changed during synchronization".to_owned());
    }
    let directory = destination_parent(&link.intent.root, &link.intent.destination)
        .map_err(|_| "destination parent could not be opened or created".to_owned())?;
    let parent = link.intent.root.join(
        link.intent
            .destination
            .parent()
            .ok_or_else(|| "destination has no parent".to_owned())?,
    );
    let opened = directory
        .metadata()
        .map_err(|_| "destination parent could not be inspected".to_owned())?;
    let observed =
        fs::symlink_metadata(&parent).map_err(|_| "destination parent changed".to_owned())?;
    if !observed.is_dir() || (opened.dev(), opened.ino()) != (observed.dev(), observed.ino()) {
        return Err("destination parent changed during synchronization".to_owned());
    }
    let name = link
        .intent
        .destination
        .file_name()
        .ok_or_else(|| "destination has no filename".to_owned())?;
    rustix::fs::symlinkat(&link.source, &directory, name)
        .map_err(|_| "link publication failed; existing destination preserved".to_owned())?;
    let published = rustix::fs::statat(&directory, name, AtFlags::SYMLINK_NOFOLLOW)
        .map_err(|_| "published link could not be inspected".to_owned())?;
    let target = rustix::fs::readlinkat(&directory, name, Vec::new())
        .map_err(|_| "published target could not be inspected".to_owned())?;
    if rustix::fs::FileType::from_raw_mode(published.st_mode) != rustix::fs::FileType::Symlink
        || target.as_bytes() != link.source.as_os_str().as_bytes()
    {
        return Err("destination changed after publication".to_owned());
    }
    validate_parent(&link.intent.root, &link.intent.destination)?;
    let observed = fs::symlink_metadata(&parent)
        .map_err(|_| "destination parent changed after publication".to_owned())?;
    if (opened.dev(), opened.ino()) != (observed.dev(), observed.ino()) {
        return Err("destination parent changed after publication".to_owned());
    }
    Ok(SyncState::Applied)
}

fn destination_parent(root: &Path, destination: &Path) -> io::Result<File> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let mut directory = File::from(rustix::fs::open(root, flags, Mode::empty())?);
    let parent = destination.parent().ok_or(io::ErrorKind::InvalidInput)?;
    for component in parent.components() {
        let Component::Normal(name) = component else {
            return Err(io::ErrorKind::InvalidInput.into());
        };
        let next = match rustix::fs::openat(&directory, name, flags, Mode::empty()) {
            Ok(next) => next,
            Err(error) if error == rustix::io::Errno::NOENT => {
                match rustix::fs::mkdirat(&directory, name, Mode::from_raw_mode(0o700)) {
                    Ok(()) => {}
                    Err(error) if error == rustix::io::Errno::EXIST => {}
                    Err(error) => return Err(error.into()),
                }
                rustix::fs::openat(&directory, name, flags, Mode::empty())?
            }
            Err(error) => return Err(error.into()),
        };
        directory = File::from(next);
    }
    Ok(directory)
}
