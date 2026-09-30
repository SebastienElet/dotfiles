use rustix::fs::{Mode, OFlags};
use std::fs::File;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path};

#[derive(Clone, Eq, PartialEq)]
pub(super) struct Snapshot {
    pub bytes: Vec<u8>,
    pub identity: (u64, u64),
}

pub(super) fn optional(root: &Path, relative: &Path) -> Result<Option<Snapshot>, &'static str> {
    let root_file = rustix::fs::open(
        root,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC,
        Mode::empty(),
    )
    .map(File::from)
    .map_err(|_| "native root cannot be safely inspected")?;
    let mut directory = root_file;
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        let Component::Normal(name) = component else {
            return Err("native path is not confined");
        };
        let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::CLOEXEC | OFlags::NONBLOCK;
        let flags = if components.peek().is_some() {
            flags | OFlags::DIRECTORY
        } else {
            flags
        };
        let opened = match rustix::fs::openat(&directory, name, flags, Mode::empty()) {
            Ok(file) => File::from(file),
            Err(rustix::io::Errno::NOENT) => return Ok(None),
            Err(_) => return Err("native path cannot be safely inspected"),
        };
        if components.peek().is_some() {
            directory = opened;
            continue;
        }
        let metadata = opened
            .metadata()
            .map_err(|_| "native metadata is unavailable")?;
        if !metadata.is_file() || metadata.nlink() != 1 || metadata.len() > 1_048_576 {
            return Err("native file must be a bounded regular file with one link");
        }
        let mut bytes = Vec::new();
        opened
            .take(1_048_577)
            .read_to_end(&mut bytes)
            .map_err(|_| "native file cannot be read")?;
        if bytes.len() > 1_048_576 {
            return Err("native file is oversized");
        }
        return Ok(Some(Snapshot {
            bytes,
            identity: (metadata.dev(), metadata.ino()),
        }));
    }
    Err("native path is empty")
}
