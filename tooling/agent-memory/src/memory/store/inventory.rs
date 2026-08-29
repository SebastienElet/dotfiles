use crate::memory::path::ManagedPath;
use crate::memory::{EntryScope, MemoryEntry, MemoryError, parse_entry};
use std::fs::File;
use std::io::Read;
use std::path::Path;

const MAX_ENTRY_BYTES: u64 = 1024 * 1024;

pub(crate) fn entry_paths(root: &ManagedPath) -> Result<Vec<ManagedPath>, MemoryError> {
    let mut paths = Vec::new();
    let user = root.join("entries/user")?;
    for name in user.read_dir_names()? {
        let name = name.to_str().ok_or_else(unsafe_path)?;
        if valid_entry_filename(name) {
            paths.push(user.join(name)?);
        }
    }
    for directory in project_directories(root)? {
        for name in directory.read_dir_names()? {
            let name = name.to_str().ok_or_else(unsafe_path)?;
            if valid_entry_filename(name) {
                paths.push(directory.join(name)?);
            }
        }
    }
    paths.sort_by(|left, right| left.relative().cmp(right.relative()));
    Ok(paths)
}

pub(super) fn repair_entry_modes(root: &ManagedPath) -> Result<(), MemoryError> {
    for directory in project_directories(root)? {
        directory.repair_directory_mode()?;
    }
    for path in entry_paths(root)? {
        path.open_read()?;
    }
    Ok(())
}

pub(super) fn validate_entry_modes(root: &ManagedPath) -> Result<(), MemoryError> {
    for directory in project_directories(root)? {
        directory.validate_directory()?;
    }
    for path in entry_paths(root)? {
        path.open_read_only()?;
    }
    Ok(())
}

fn project_directories(root: &ManagedPath) -> Result<Vec<ManagedPath>, MemoryError> {
    let projects = root.join("entries/project")?;
    projects
        .read_dir_names()?
        .into_iter()
        .map(|key| {
            let key = key.to_str().ok_or_else(unsafe_path)?;
            if !valid_project_key(key) {
                return Err(unsafe_path());
            }
            projects.join(key)
        })
        .collect()
}

pub(crate) fn read_entry(path: &ManagedPath) -> Result<MemoryEntry, MemoryError> {
    let mut file = path.open_read_only()?;
    read_entry_from_file(path, &mut file)
}

pub(crate) fn read_entry_from_file(
    path: &ManagedPath,
    file: &mut File,
) -> Result<MemoryEntry, MemoryError> {
    let bytes = read_bounded_file(file)?;
    let entry = parse_entry(&bytes)?;
    validate_entry_location(path.relative(), &entry)?;
    Ok(entry)
}

fn read_bounded_file(file: &mut File) -> Result<Vec<u8>, MemoryError> {
    let metadata = file.metadata().map_err(store_io)?;
    if metadata.len() > MAX_ENTRY_BYTES {
        return Err(store_error());
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(file)
        .take(MAX_ENTRY_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(store_io)?;
    if bytes.len() as u64 > MAX_ENTRY_BYTES {
        return Err(store_error());
    }
    Ok(bytes)
}

fn validate_entry_location(path: &Path, entry: &MemoryEntry) -> Result<(), MemoryError> {
    let components = path
        .iter()
        .map(|component| component.to_str().ok_or_else(unsafe_path))
        .collect::<Result<Vec<_>, _>>()?;
    let filename = components.last().ok_or_else(unsafe_path)?;
    let id = filename.strip_suffix(".yaml").ok_or_else(unsafe_path)?;
    if id != entry.id().as_str() {
        return Err(MemoryError::unavailable("entry_path_mismatch", "id"));
    }
    match (components.as_slice(), entry.scope()) {
        (["entries", "user", _], EntryScope::User) => Ok(()),
        (["entries", "project", key, _], EntryScope::Project(entry_key))
            if *key == entry_key.as_str() =>
        {
            Ok(())
        }
        _ => Err(MemoryError::unavailable("entry_path_mismatch", "scope")),
    }
}

fn valid_entry_filename(value: &str) -> bool {
    value.strip_suffix(".yaml").is_some_and(valid_memory_id)
}

pub(crate) fn valid_memory_id(value: &str) -> bool {
    value
        .strip_prefix("mem_")
        .is_some_and(|suffix| suffix.len() == 24 && suffix.bytes().all(is_lower_hex))
}

pub(crate) fn valid_project_key(value: &str) -> bool {
    value
        .strip_prefix("project_")
        .is_some_and(|suffix| suffix.len() == 64 && suffix.bytes().all(is_lower_hex))
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn store_io(_: std::io::Error) -> MemoryError {
    store_error()
}

const fn store_error() -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::unavailable("unsafe_store_path", "store")
}
