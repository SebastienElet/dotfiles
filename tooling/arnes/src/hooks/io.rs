use super::HooksError;
use rustix::fs::{AtFlags, Mode, OFlags, RenameFlags};
use std::fs::File;
use std::io::{Read, Write};
use std::os::unix::fs::MetadataExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    io,
    path::{Component, Path, PathBuf},
};

#[cfg(test)]
mod test_hook;
#[cfg(test)]
mod tests;
#[cfg(test)]
use test_hook::{run_after_publish_hook, set_after_publish_hook};

const MAX_CONFIG_BYTES: usize = 1_048_576;
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Eq, PartialEq)]
struct Snapshot {
    bytes: Vec<u8>,
    device: u64,
    inode: u64,
    mode: Mode,
}

pub struct ConfigFile {
    home: File,
    home_path: PathBuf,
    directory_components: Vec<String>,
    directory: File,
    name: String,
    original: Option<Snapshot>,
    _lock: File,
}

impl ConfigFile {
    pub fn open(home: &Path, agent_directory: &str, name: &str) -> Result<Self, HooksError> {
        Self::open_path(home, &Path::new(agent_directory).join(name))
    }

    pub fn open_path(home: &Path, path: &Path) -> Result<Self, HooksError> {
        let mut components = confined_components(path)?;
        let name = components
            .pop()
            .ok_or_else(|| HooksError::new("configuration path is empty"))?;
        let home_path = home.to_owned();
        let home = open_directory(home)?;
        let directory = descend(&home, &components, true)?;
        let lock = open_lock(&directory, &format!(".{name}.lock"))?;
        lock.lock()?;
        let mut file = Self {
            home,
            home_path,
            directory_components: components,
            directory,
            name,
            original: None,
            _lock: lock,
        };
        file.check_directory()?;
        file.original = read_at(&file.directory, &file.name)?;
        Ok(file)
    }

    pub fn content(&self) -> Option<&[u8]> {
        self.original
            .as_ref()
            .map(|snapshot| snapshot.bytes.as_slice())
    }

    pub fn identity(&self) -> Option<(u64, u64)> {
        self.original
            .as_ref()
            .map(|snapshot| (snapshot.device, snapshot.inode))
    }

    pub fn replace(self, bytes: &[u8]) -> Result<(), HooksError> {
        self.replace_with_identity(bytes).map(|_| ())
    }

    pub fn replace_with_identity(self, bytes: &[u8]) -> Result<(u64, u64), HooksError> {
        self.check_directory()?;
        if let Some(original) = self.original.as_ref()
            && original.bytes == bytes
        {
            if read_at(&self.directory, &self.name)?.as_ref() == self.original.as_ref() {
                return Ok((original.device, original.inode));
            }
            return Err(HooksError::new(
                "hook configuration changed during installation",
            ));
        }
        if bytes.len() > MAX_CONFIG_BYTES {
            return Err(HooksError::new("hook configuration is oversized"));
        }
        let temporary = temporary_name(&self.name);
        let mode = self
            .original
            .as_ref()
            .map_or(Mode::from_raw_mode(0o600), |value| value.mode);
        let expected = match write_temporary(&self.directory, &temporary, bytes, mode) {
            Ok(expected) => expected,
            Err(error) => {
                let _ = rustix::fs::unlinkat(&self.directory, &temporary, AtFlags::empty());
                return Err(error);
            }
        };
        let result = self.commit(&temporary, &expected);
        if result.is_err() {
            let _ = rustix::fs::unlinkat(&self.directory, &temporary, AtFlags::empty());
        }
        result.map(|()| (expected.device, expected.inode))
    }

    fn commit(&self, temporary: &str, expected: &Snapshot) -> Result<(), HooksError> {
        self.check_directory()?;
        match &self.original {
            None => rename_new(&self.directory, temporary, &self.name)?,
            Some(original) => replace_existing(&self.directory, temporary, &self.name, original)?,
        }
        run_after_publish_hook();
        self.check_directory()?;
        if read_at(&self.directory, &self.name)?.as_ref() != Some(expected) {
            return Err(changed());
        }
        if self.original.is_some() {
            rustix::fs::unlinkat(&self.directory, temporary, AtFlags::empty()).map_err(errno)?;
        }
        rustix::fs::fsync(&self.directory).map_err(errno)?;
        Ok(())
    }

    fn check_directory(&self) -> Result<(), HooksError> {
        let home = open_directory(&self.home_path).map_err(|_| changed())?;
        let directory = descend(&home, &self.directory_components, false).map_err(|_| changed())?;
        if !same_directory(&home, &self.home)? || !same_directory(&directory, &self.directory)? {
            return Err(changed());
        }
        Ok(())
    }
}

fn confined_components(path: &Path) -> Result<Vec<String>, HooksError> {
    path.components()
        .filter(|component| *component != Component::CurDir)
        .map(|component| match component {
            Component::Normal(name) => name
                .to_str()
                .map(str::to_owned)
                .ok_or_else(|| HooksError::new("configuration path is not valid UTF-8")),
            Component::Prefix(_)
            | Component::RootDir
            | Component::CurDir
            | Component::ParentDir => Err(HooksError::new(
                "configuration path must remain inside its root",
            )),
        })
        .collect()
}

fn descend(root: &File, components: &[String], create: bool) -> Result<File, HooksError> {
    let mut directory = root.try_clone()?;
    for component in components {
        if create {
            create_directory(&directory, component)?;
        }
        directory = open_directory_at(&directory, component)?;
    }
    Ok(directory)
}

fn same_directory(left: &File, right: &File) -> Result<bool, HooksError> {
    let left = left.metadata()?;
    let right = right.metadata()?;
    Ok(left.dev() == right.dev() && left.ino() == right.ino())
}

fn open_directory(path: &Path) -> Result<File, HooksError> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let file = rustix::fs::open(path, flags, Mode::empty()).map_err(errno)?;
    Ok(File::from(file))
}

fn create_directory(home: &File, name: &str) -> Result<(), HooksError> {
    match rustix::fs::mkdirat(home, name, Mode::from_raw_mode(0o700)) {
        Ok(()) => Ok(()),
        Err(error) if error == rustix::io::Errno::EXIST => Ok(()),
        Err(error) => Err(errno(error)),
    }
}

fn open_directory_at(home: &File, name: &str) -> Result<File, HooksError> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let file = rustix::fs::openat(home, name, flags, Mode::empty()).map_err(errno)?;
    Ok(File::from(file))
}

fn open_lock(directory: &File, name: &str) -> Result<File, HooksError> {
    let flags = OFlags::RDWR | OFlags::CREATE | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let file =
        rustix::fs::openat(directory, name, flags, Mode::from_raw_mode(0o600)).map_err(errno)?;
    let file = File::from(file);
    validate_regular(&file)?;
    rustix::fs::fchmod(&file, Mode::from_raw_mode(0o600)).map_err(errno)?;
    Ok(file)
}

fn read_at(directory: &File, name: &str) -> Result<Option<Snapshot>, HooksError> {
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let file = match rustix::fs::openat(directory, name, flags, Mode::empty()) {
        Ok(file) => file,
        Err(error) if error == rustix::io::Errno::NOENT => return Ok(None),
        Err(error) => return Err(errno(error)),
    };
    let mut file = File::from(file);
    let metadata = validate_regular(&file)?;
    let mut bytes = Vec::new();
    Read::by_ref(&mut file)
        .take((MAX_CONFIG_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_CONFIG_BYTES {
        return Err(HooksError::new("hook configuration is oversized"));
    }
    Ok(Some(snapshot(bytes, &metadata)))
}

fn validate_regular(file: &File) -> Result<std::fs::Metadata, HooksError> {
    let metadata = file.metadata()?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(HooksError::new(
            "hook configuration path must be a single-link regular file",
        ));
    }
    Ok(metadata)
}

fn write_temporary(
    directory: &File,
    name: &str,
    bytes: &[u8],
    mode: Mode,
) -> Result<Snapshot, HooksError> {
    let flags = OFlags::WRONLY | OFlags::CREATE | OFlags::EXCL | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    let file =
        rustix::fs::openat(directory, name, flags, Mode::from_raw_mode(0o600)).map_err(errno)?;
    let mut file = File::from(file);
    rustix::fs::fchmod(&file, mode).map_err(errno)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    let metadata = validate_regular(&file)?;
    Ok(snapshot(bytes.to_vec(), &metadata))
}

fn snapshot(bytes: Vec<u8>, metadata: &std::fs::Metadata) -> Snapshot {
    Snapshot {
        bytes,
        device: metadata.dev(),
        inode: metadata.ino(),
        mode: Mode::from_raw_mode((metadata.mode() & 0o777) as _),
    }
}

fn rename_new(directory: &File, temporary: &str, name: &str) -> Result<(), HooksError> {
    rustix::fs::renameat_with(
        directory,
        temporary,
        directory,
        name,
        RenameFlags::NOREPLACE,
    )
    .map_err(errno)
}

fn replace_existing(
    directory: &File,
    temporary: &str,
    name: &str,
    original: &Snapshot,
) -> Result<(), HooksError> {
    rustix::fs::renameat_with(directory, temporary, directory, name, RenameFlags::EXCHANGE)
        .map_err(errno)?;
    let current = read_at(directory, temporary);
    if current.as_ref().ok().and_then(Option::as_ref) == Some(original) {
        return Ok(());
    }
    rustix::fs::renameat_with(directory, temporary, directory, name, RenameFlags::EXCHANGE)
        .map_err(errno)?;
    current?;
    Err(changed())
}

#[cfg(not(test))]
const fn run_after_publish_hook() {}

fn changed() -> HooksError {
    HooksError::new("hook configuration changed during installation")
}

fn temporary_name(name: &str) -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    format!(".{name}.tmp-{}-{timestamp}-{sequence}", std::process::id())
}

fn errno(error: rustix::io::Errno) -> HooksError {
    io::Error::from(error).into()
}
