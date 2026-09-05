use super::super::MeasureError;
use rustix::fs::{AtFlags, Mode, OFlags};
use std::ffi::{OsStr, OsString};
use std::fs::File;
use std::os::unix::ffi::OsStrExt;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

mod removal;

#[derive(Clone)]
pub struct ManagedPath {
    root: Arc<File>,
    relative: PathBuf,
    display: PathBuf,
}

impl ManagedPath {
    pub fn root(root: File, display: PathBuf) -> Self {
        Self {
            root: Arc::new(root),
            relative: PathBuf::new(),
            display,
        }
    }

    #[cfg(test)]
    pub fn test_path(path: &Path) -> Self {
        let parent = path.parent().unwrap();
        let root = rustix::fs::open(
            parent,
            OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
            Mode::empty(),
        )
        .unwrap();
        Self::root(root.into(), parent.to_owned()).join(path.file_name().unwrap())
    }

    pub fn join(&self, path: impl AsRef<Path>) -> Self {
        Self {
            root: Arc::clone(&self.root),
            relative: self.relative.join(path.as_ref()),
            display: self.display.join(path.as_ref()),
        }
    }

    pub fn join_extension(&self, extension: impl AsRef<OsStr>) -> Self {
        let mut relative = self.relative.clone();
        let mut display = self.display.clone();
        relative.set_extension(extension.as_ref());
        display.set_extension(extension.as_ref());
        Self {
            root: Arc::clone(&self.root),
            relative,
            display,
        }
    }

    pub fn create_dir_all(&self) -> Result<(), MeasureError> {
        let mut directory = self.root.try_clone()?;
        for component in normal_components(&self.relative)? {
            match rustix::fs::mkdirat(&directory, component, private_dir_mode()) {
                Ok(()) => {}
                Err(rustix::io::Errno::EXIST) => {}
                Err(error) => return Err(std::io::Error::from(error).into()),
            }
            directory = open_directory_at(&directory, component, &self.display)?;
            rustix::fs::fchmod(&directory, private_dir_mode())?;
        }
        Ok(())
    }

    pub fn open_directory(&self) -> Result<File, MeasureError> {
        let mut directory = self.root.try_clone()?;
        for component in normal_components(&self.relative)? {
            directory = open_directory_at(&directory, component, &self.display)?;
        }
        Ok(directory)
    }

    pub fn open_read(&self) -> Result<File, MeasureError> {
        let (directory, name) = self.parent_and_name()?;
        let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
        let file = File::from(rustix::fs::openat(directory, name, flags, Mode::empty())?);
        ensure_single_link(&file, &self.display)?;
        Ok(file)
    }

    pub fn exists(&self) -> Result<bool, MeasureError> {
        let mut components = normal_components(&self.relative)?;
        let Some(name) = components.pop() else {
            return Ok(true);
        };
        let mut directory = self.root.try_clone()?;
        for component in components {
            let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
            match rustix::fs::openat(&directory, component, flags, Mode::empty()) {
                Ok(next) => directory = next.into(),
                Err(rustix::io::Errno::NOENT) => return Ok(false),
                Err(error) => return Err(std::io::Error::from(error).into()),
            }
        }
        match rustix::fs::statat(directory, name, AtFlags::SYMLINK_NOFOLLOW) {
            Ok(_) => Ok(true),
            Err(rustix::io::Errno::NOENT) => Ok(false),
            Err(error) => Err(std::io::Error::from(error).into()),
        }
    }

    pub fn open_append(&self) -> Result<File, MeasureError> {
        let (directory, name) = self.parent_and_name()?;
        let create = OFlags::RDWR
            | OFlags::CREATE
            | OFlags::EXCL
            | OFlags::APPEND
            | OFlags::NOFOLLOW
            | OFlags::CLOEXEC;
        let file = match rustix::fs::openat(&directory, &name, create, private_file_mode()) {
            Ok(file) => File::from(file),
            Err(rustix::io::Errno::EXIST) => {
                let open = OFlags::RDWR | OFlags::APPEND | OFlags::NOFOLLOW | OFlags::CLOEXEC;
                File::from(rustix::fs::openat(directory, name, open, Mode::empty())?)
            }
            Err(error) => return Err(error.into()),
        };
        ensure_single_link(&file, &self.display)?;
        rustix::fs::fchmod(&file, private_file_mode())?;
        Ok(file)
    }

    pub fn open_new(&self) -> Result<File, MeasureError> {
        let (directory, name) = self.parent_and_name()?;
        let flags =
            OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC;
        let file = File::from(rustix::fs::openat(
            directory,
            name,
            flags,
            private_file_mode(),
        )?);
        rustix::fs::fchmod(&file, private_file_mode())?;
        Ok(file)
    }

    pub fn rename_to(&self, destination: &Self) -> Result<(), MeasureError> {
        let (source_dir, source) = self.parent_and_name()?;
        let (destination_dir, destination) = destination.parent_and_name()?;
        rustix::fs::renameat(&source_dir, source, &destination_dir, destination)?;
        Ok(())
    }

    pub fn remove_file(&self) -> Result<(), MeasureError> {
        let (directory, name) = self.parent_and_name()?;
        rustix::fs::unlinkat(directory, name, AtFlags::empty())?;
        Ok(())
    }

    pub fn read_dir_names(&self) -> Result<Vec<OsString>, MeasureError> {
        let directory = self.open_directory()?;
        let mut names = Vec::new();
        for entry in rustix::fs::Dir::read_from(directory)? {
            let entry = entry?;
            let name = entry.file_name().to_bytes();
            if name != b"." && name != b".." {
                names.push(OsStr::from_bytes(name).to_owned());
            }
        }
        Ok(names)
    }

    fn parent_and_name(&self) -> Result<(File, OsString), MeasureError> {
        let mut components = normal_components(&self.relative)?;
        let name = components
            .pop()
            .ok_or_else(|| MeasureError::new("managed file path has no name"))?;
        let mut directory = self.root.try_clone()?;
        for component in components {
            directory = open_directory_at(&directory, component, &self.display)?;
        }
        Ok((directory, name.to_owned()))
    }
}

pub fn open_root(path: &Path) -> Result<File, MeasureError> {
    let components = normal_components(path)?;
    let mut directory = File::from(rustix::fs::open(
        "/",
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )?);
    let managed_start = components.len().saturating_sub(2);
    for (index, component) in components.into_iter().enumerate() {
        match rustix::fs::mkdirat(&directory, component, private_dir_mode()) {
            Ok(()) | Err(rustix::io::Errno::EXIST) => {}
            Err(error) => return Err(std::io::Error::from(error).into()),
        }
        directory = open_directory_at(&directory, component, path)?;
        if index >= managed_start {
            rustix::fs::fchmod(&directory, private_dir_mode())?;
        }
    }
    Ok(directory)
}

fn open_directory_at(directory: &File, name: &OsStr, display: &Path) -> Result<File, MeasureError> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    rustix::fs::openat(directory, name, flags, Mode::empty())
        .map(File::from)
        .map_err(|error| {
            MeasureError::new(format!(
                "managed path is not a real directory: {}: {}",
                display.display(),
                std::io::Error::from(error)
            ))
        })
}

fn normal_components(path: &Path) -> Result<Vec<&OsStr>, MeasureError> {
    path.components()
        .filter_map(|component| match component {
            Component::RootDir => None,
            Component::Normal(value) => Some(Ok(value)),
            _ => Some(Err(MeasureError::new(
                "managed path has an unsafe component",
            ))),
        })
        .collect()
}

fn ensure_single_link(file: &File, path: &Path) -> Result<(), MeasureError> {
    let metadata = file.metadata()?;
    if !metadata.is_file() || std::os::unix::fs::MetadataExt::nlink(&metadata) != 1 {
        return Err(MeasureError::new(format!(
            "managed path is not a single-link regular file: {}",
            path.display()
        )));
    }
    Ok(())
}

fn private_dir_mode() -> Mode {
    Mode::from_raw_mode(0o700)
}

fn private_file_mode() -> Mode {
    Mode::from_raw_mode(0o600)
}
