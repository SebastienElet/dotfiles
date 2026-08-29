use super::{MemoryRoot, unsafe_path};
use crate::memory::MemoryError;
use rustix::fs::{Mode, OFlags};
use std::ffi::OsStr;
use std::fs::File;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path};

pub(crate) fn open_root(root: &MemoryRoot, fail_mode_repair: bool) -> Result<File, MemoryError> {
    let components = normal_components(root.path())?;
    if components.is_empty() {
        return Err(unsafe_path());
    }
    let mut directory = rustix::fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(classify_path_error)?;
    for component in components {
        match rustix::fs::mkdirat(&directory, component, private_directory_mode()) {
            Ok(()) | Err(rustix::io::Errno::EXIST) => {}
            Err(error) => return Err(classify_path_error(error)),
        }
        directory = open_directory_at(&directory, component)?;
    }
    repair_mode(&directory, private_directory_mode(), fail_mode_repair)?;
    Ok(directory)
}

pub(crate) fn open_existing_root(root: &MemoryRoot) -> Result<Option<File>, MemoryError> {
    match std::fs::symlink_metadata(root.path()) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(store_unavailable(error)),
    }
    let mut directory = rustix::fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(classify_path_error)?;
    for component in normal_components(root.path())? {
        directory = open_directory_at(&directory, component)?;
    }
    validate_mode(&directory, private_directory_mode())?;
    Ok(Some(directory))
}

pub(super) fn open_directory_at(directory: &File, name: &OsStr) -> Result<File, MemoryError> {
    rustix::fs::openat(
        directory,
        name,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(classify_path_error)
}

pub(super) fn repair_mode(file: &File, mode: Mode, fail: bool) -> Result<(), MemoryError> {
    if fail {
        return Err(MemoryError::unavailable(
            "store_permissions_unavailable",
            "store",
        ));
    }
    rustix::fs::fchmod(file, mode)
        .map_err(|_| MemoryError::unavailable("store_permissions_unavailable", "store"))
}

pub(super) fn validate_mode(file: &File, expected: Mode) -> Result<(), MemoryError> {
    let actual = file.metadata().map_err(store_unavailable)?.mode() & 0o777;
    if actual == u32::from(expected.as_raw_mode()) {
        Ok(())
    } else {
        Err(MemoryError::unavailable(
            "store_permissions_unavailable",
            "store",
        ))
    }
}

pub(super) fn ensure_single_link_regular(file: &File) -> Result<(), MemoryError> {
    let metadata = file.metadata().map_err(store_unavailable)?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(unsafe_path());
    }
    Ok(())
}

pub(super) fn normal_components(path: &Path) -> Result<Vec<&OsStr>, MemoryError> {
    path.components()
        .filter_map(|component| match component {
            Component::RootDir => None,
            Component::Normal(value) => Some(Ok(value)),
            _ => Some(Err(unsafe_path())),
        })
        .collect()
}

pub(super) fn classify_path_error(error: rustix::io::Errno) -> MemoryError {
    if matches!(
        error,
        rustix::io::Errno::LOOP | rustix::io::Errno::NOTDIR | rustix::io::Errno::ISDIR
    ) {
        unsafe_path()
    } else {
        store_unavailable(std::io::Error::from(error))
    }
}

pub(super) fn store_unavailable(_: std::io::Error) -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}

pub(super) const fn private_directory_mode() -> Mode {
    Mode::from_raw_mode(0o700)
}

pub(super) const fn private_file_mode() -> Mode {
    Mode::from_raw_mode(0o600)
}
