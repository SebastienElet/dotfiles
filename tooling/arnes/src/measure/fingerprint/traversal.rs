use super::SelectedRoot;
use crate::measure::MeasureError;
use sha2::{Digest, Sha256};
use std::collections::{BinaryHeap, HashSet};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 512;
const MAX_FILE_BYTES: u64 = 1_048_576;
const WINDOW_BYTES: u64 = 65_536;

pub struct Traversal<'a> {
    hasher: &'a mut Sha256,
    limitations: &'a mut Vec<String>,
    seen: HashSet<PathBuf>,
    remaining: usize,
}

impl<'a> Traversal<'a> {
    pub fn new(hasher: &'a mut Sha256, limitations: &'a mut Vec<String>) -> Self {
        Self {
            hasher,
            limitations,
            seen: HashSet::new(),
            remaining: MAX_FILES,
        }
    }

    pub fn hash_markers(&mut self, markers: &[String]) -> Result<(), MeasureError> {
        for marker in markers {
            if self.consume("plugin marker")? {
                write!(self.hasher, "plugin-state\0{marker}\0")?;
            }
        }
        Ok(())
    }

    pub fn hash_selected(&mut self, root: &SelectedRoot) -> Result<(), MeasureError> {
        let boundary = if root.bounded && root.path.exists() {
            Some(fs::canonicalize(&root.path)?)
        } else {
            None
        };
        if let Err(error) = self.hash_path(&root.path, &root.label, boundary.as_deref()) {
            write!(self.hasher, "unreadable\0{:?}\0", error.kind())?;
        }
        Ok(())
    }

    fn hash_path(
        &mut self,
        path: &Path,
        label: &Path,
        boundary: Option<&Path>,
    ) -> std::io::Result<()> {
        write!(self.hasher, "{}\0", label.display())?;
        if !path.exists() {
            self.hasher.update(b"missing\0");
            return Ok(());
        }
        if !self.consume(&label.display().to_string())? {
            self.hasher.update(b"omitted\0");
            return Ok(());
        }
        let canonical = fs::canonicalize(path)?;
        if boundary.is_some_and(|boundary| !canonical.starts_with(boundary)) {
            self.hasher.update(b"escape\0");
            return Ok(());
        }
        if !self.seen.insert(canonical) {
            self.hasher.update(b"cycle\0");
            return Ok(());
        }
        let metadata = fs::metadata(path)?;
        if metadata.is_dir() {
            self.hash_directory(path, label, boundary)
        } else if metadata.is_file() {
            self.hash_file(path, metadata.len())
        } else {
            self.hasher.update(b"unsupported\0");
            Ok(())
        }
    }

    fn hash_directory(
        &mut self,
        path: &Path,
        label: &Path,
        boundary: Option<&Path>,
    ) -> std::io::Result<()> {
        self.hasher.update(b"directory\0");
        let inventory = bounded_entries(path, self.remaining)?;
        if inventory.total > inventory.selected.len() {
            write!(self.hasher, "truncated\0{}\0", inventory.total)?;
            self.hasher.update(inventory.aggregate);
            self.limit("fingerprint inventory is lexicographically bounded to 512 entries");
        }
        for (name, path) in inventory.selected {
            self.hash_path(&path, &label.join(name), boundary)?;
        }
        Ok(())
    }

    fn hash_file(&mut self, path: &Path, size: u64) -> std::io::Result<()> {
        write!(self.hasher, "file\0{size}\0")?;
        let mut file = File::open(path)?;
        if size <= MAX_FILE_BYTES {
            std::io::copy(&mut file, self.hasher)?;
            return Ok(());
        }
        std::io::copy(
            &mut std::io::Read::by_ref(&mut file).take(WINDOW_BYTES),
            self.hasher,
        )?;
        file.seek(SeekFrom::End(-(WINDOW_BYTES as i64)))?;
        std::io::copy(&mut file.take(WINDOW_BYTES), self.hasher)?;
        self.limit("fingerprint oversized files use size and 65536-byte boundary windows");
        Ok(())
    }

    fn consume(&mut self, label: &str) -> std::io::Result<bool> {
        if self.remaining == 0 {
            write!(self.hasher, "inventory-omitted\0{label}\0")?;
            self.limit("fingerprint inventory is lexicographically bounded to 512 entries");
            return Ok(false);
        }
        self.remaining -= 1;
        Ok(true)
    }

    fn limit(&mut self, limitation: &str) {
        if !self.limitations.iter().any(|value| value == limitation) {
            self.limitations.push(limitation.to_owned());
        }
    }
}

struct DirectoryEntries {
    selected: Vec<(OsString, PathBuf)>,
    total: usize,
    aggregate: [u8; 32],
}

fn bounded_entries(path: &Path, limit: usize) -> std::io::Result<DirectoryEntries> {
    let mut entries = BinaryHeap::new();
    let mut total = 0;
    let mut names = [0_u8; 32];
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let item = (entry.file_name(), entry.path());
        let digest = Sha256::digest(item.0.as_bytes());
        for (aggregate, byte) in names.iter_mut().zip(digest) {
            *aggregate ^= byte;
        }
        total += 1;
        entries.push(item);
        if entries.len() > limit {
            entries.pop();
        }
    }
    let mut entries = entries.into_vec();
    entries.sort();
    Ok(DirectoryEntries {
        selected: entries,
        total,
        aggregate: names,
    })
}
