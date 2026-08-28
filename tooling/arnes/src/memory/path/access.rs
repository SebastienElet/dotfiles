use super::component::{
    classify_path_error, ensure_single_link_regular, normal_components, open_directory_at,
    private_directory_mode, private_file_mode, repair_mode, store_unavailable,
};
use super::unsafe_path;
use crate::memory::MemoryError;
use rustix::fs::{AtFlags, Mode, OFlags, RenameFlags};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub(crate) struct ManagedPath {
    root: Arc<File>,
    relative: PathBuf,
}

impl ManagedPath {
    pub(crate) fn root(root: File) -> Self {
        Self {
            root: Arc::new(root),
            relative: PathBuf::new(),
        }
    }

    pub(crate) fn join(&self, path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        normal_components(path.as_ref())?;
        Ok(Self {
            root: Arc::clone(&self.root),
            relative: self.relative.join(path),
        })
    }

    pub(crate) fn open_root_directory(&self) -> Result<File, MemoryError> {
        rustix::fs::openat(
            &self.root,
            ".",
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .map(File::from)
        .map_err(classify_path_error)
    }

    pub(crate) fn ensure_directory(&self, fail_mode_repair: bool) -> Result<(), MemoryError> {
        let mut directory = self.root.try_clone().map_err(store_unavailable)?;
        for component in normal_components(&self.relative)? {
            match rustix::fs::mkdirat(&directory, component, private_directory_mode()) {
                Ok(()) | Err(rustix::io::Errno::EXIST) => {}
                Err(error) => return Err(classify_path_error(error)),
            }
            directory = open_directory_at(&directory, component)?;
            repair_mode(&directory, private_directory_mode(), fail_mode_repair)?;
        }
        Ok(())
    }

    pub(crate) fn ensure_file(&self, fail_mode_repair: bool) -> Result<File, MemoryError> {
        let (directory, name) = self.parent_and_name()?;
        let flags =
            OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
        let file = rustix::fs::openat(directory, name, flags, private_file_mode())
            .map(File::from)
            .map_err(classify_path_error)?;
        ensure_single_link_regular(&file)?;
        repair_mode(&file, private_file_mode(), fail_mode_repair)?;
        Ok(file)
    }

    pub(crate) fn open_existing_read_write(&self) -> Result<File, MemoryError> {
        self.open_existing(OFlags::RDWR)
    }

    pub(crate) fn repair_private_file_mode(&self, file: &File) -> Result<(), MemoryError> {
        ensure_single_link_regular(file)?;
        repair_mode(file, private_file_mode(), false)
    }

    pub(crate) fn ensure_same_file(&self, expected: &File) -> Result<(), MemoryError> {
        let current = self.open_existing(OFlags::RDONLY)?;
        let expected_metadata = expected.metadata().map_err(store_unavailable)?;
        let current_metadata = current.metadata().map_err(store_unavailable)?;
        if expected_metadata.dev() != current_metadata.dev()
            || expected_metadata.ino() != current_metadata.ino()
            || expected_metadata.nlink() != 1
        {
            return Err(unsafe_path());
        }
        Ok(())
    }

    pub(crate) fn open_read(&self) -> Result<File, MemoryError> {
        let file = self.open_existing(OFlags::RDONLY)?;
        self.repair_private_file_mode(&file)?;
        Ok(file)
    }

    pub(crate) fn open_new(&self) -> Result<File, MemoryError> {
        let (directory, name) = self.parent_and_name()?;
        let flags =
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        let file = rustix::fs::openat(directory, name, flags, private_file_mode())
            .map(File::from)
            .map_err(classify_path_error)?;
        ensure_single_link_regular(&file)?;
        repair_mode(&file, private_file_mode(), false)?;
        Ok(file)
    }

    pub(crate) fn exists(&self) -> Result<bool, MemoryError> {
        let (directory, name) = self.parent_and_name()?;
        match rustix::fs::statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(_) => Ok(true),
            Err(rustix::io::Errno::NOENT) => Ok(false),
            Err(error) => Err(classify_path_error(error)),
        }
    }

    pub(crate) fn read_dir_names(&self) -> Result<Vec<OsString>, MemoryError> {
        let directory = self.open_directory()?;
        let mut names = Vec::new();
        for entry in rustix::fs::Dir::read_from(directory).map_err(classify_path_error)? {
            let entry = entry.map_err(classify_path_error)?;
            let name = entry.file_name().to_bytes();
            if name != b"." && name != b".." {
                names.push(OsStr::from_bytes(name).to_owned());
            }
        }
        Ok(names)
    }

    pub(crate) fn rename_new_to(&self, destination: &Self) -> Result<(), MemoryError> {
        let (source_directory, source) = self.parent_and_name()?;
        let (destination_directory, destination) = destination.parent_and_name()?;
        rustix::fs::renameat_with(
            source_directory,
            source,
            destination_directory,
            destination,
            RenameFlags::NOREPLACE,
        )
        .map_err(classify_path_error)
    }

    pub(crate) fn rename_to(&self, destination: &Self) -> Result<(), MemoryError> {
        let (source_directory, source) = self.parent_and_name()?;
        let (destination_directory, destination) = destination.parent_and_name()?;
        rustix::fs::renameat(source_directory, source, destination_directory, destination)
            .map_err(classify_path_error)
    }

    pub(crate) fn sync_directory(&self) -> Result<(), MemoryError> {
        rustix::fs::fsync(self.open_directory()?).map_err(classify_path_error)
    }

    pub(crate) fn sync_parent_directory(&self) -> Result<(), MemoryError> {
        let (directory, _) = self.parent_and_name()?;
        rustix::fs::fsync(directory).map_err(classify_path_error)
    }

    pub(crate) fn relative(&self) -> &Path {
        &self.relative
    }

    fn open_existing(&self, access: OFlags) -> Result<File, MemoryError> {
        let (directory, name) = self.parent_and_name()?;
        let flags = access | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
        let file = rustix::fs::openat(directory, name, flags, Mode::empty())
            .map(File::from)
            .map_err(classify_path_error)?;
        ensure_single_link_regular(&file)?;
        Ok(file)
    }

    fn open_directory(&self) -> Result<File, MemoryError> {
        let mut directory = self.root.try_clone().map_err(store_unavailable)?;
        for component in normal_components(&self.relative)? {
            directory = open_directory_at(&directory, component)?;
            repair_mode(&directory, private_directory_mode(), false)?;
        }
        Ok(directory)
    }

    fn parent_and_name(&self) -> Result<(File, OsString), MemoryError> {
        let mut components = normal_components(&self.relative)?;
        let name = components.pop().ok_or_else(unsafe_path)?.to_owned();
        let mut directory = self.root.try_clone().map_err(store_unavailable)?;
        for component in components {
            directory = open_directory_at(&directory, component)?;
            repair_mode(&directory, private_directory_mode(), false)?;
        }
        Ok((directory, name))
    }
}
