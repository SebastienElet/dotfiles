use super::document::relative_string;
use crate::memory::path::ManagedPath;
use crate::memory::store::inventory::{entry_paths, read_entry_from_file};
use crate::memory::{MemoryEntry, MemoryError};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs::{File, Metadata};
use std::os::unix::fs::MetadataExt;

const INDEX_HEADER_MAX_BYTES: u64 = 4096;
const INDEX_ITEM_MAX_BYTES: u64 = 256 * 1024;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(super) struct InventoryItem {
    pub(super) path: String,
    pub(super) length: u64,
    pub(super) modified_ns: i64,
    #[serde(skip)]
    pub(super) device: u64,
    #[serde(skip)]
    pub(super) inode: u64,
}

pub(super) struct AnchoredEntry {
    pub(super) path: ManagedPath,
    file: File,
    item: InventoryItem,
}

pub(crate) struct InventorySnapshot {
    entries: Vec<AnchoredEntry>,
}

impl InventorySnapshot {
    pub(super) fn capture(root: &ManagedPath) -> Result<Self, MemoryError> {
        Self::from_paths(&entry_paths(root)?)
    }

    fn from_paths(paths: &[ManagedPath]) -> Result<Self, MemoryError> {
        let mut entries = paths
            .iter()
            .map(AnchoredEntry::open)
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort_by(|left, right| left.item.path.cmp(&right.item.path));
        Ok(Self { entries })
    }

    pub(super) fn items(&self) -> Vec<InventoryItem> {
        self.entries
            .iter()
            .map(|entry| entry.item.clone())
            .collect()
    }

    pub(super) fn entries_mut(&mut self) -> &mut [AnchoredEntry] {
        &mut self.entries
    }

    pub(super) fn item_for(&self, path: &ManagedPath) -> Option<&InventoryItem> {
        self.entries
            .iter()
            .find(|entry| entry.path.relative() == path.relative())
            .map(AnchoredEntry::item)
    }

    pub(super) fn replace(
        &mut self,
        destination: &ManagedPath,
        file: File,
    ) -> Result<(), MemoryError> {
        self.entries
            .retain(|entry| entry.path.relative() != destination.relative());
        self.entries
            .push(AnchoredEntry::from_file(destination.clone(), file)?);
        self.entries
            .sort_by(|left, right| left.item.path.cmp(&right.item.path));
        Ok(())
    }

    pub(super) fn matches_current(&self, root: &ManagedPath) -> Result<bool, MemoryError> {
        let current = Self::capture(root)?;
        if self.items() != current.items() {
            return Ok(false);
        }
        self.entries
            .iter()
            .try_for_each(AnchoredEntry::ensure_current)?;
        Ok(true)
    }

    pub(crate) fn ensure_current(&self, root: &ManagedPath) -> Result<(), MemoryError> {
        if self.matches_current(root)? {
            Ok(())
        } else {
            Err(unsafe_path())
        }
    }
}

impl AnchoredEntry {
    fn open(path: &ManagedPath) -> Result<Self, MemoryError> {
        Self::from_file(path.clone(), path.open_read()?)
    }

    fn from_file(path: ManagedPath, file: File) -> Result<Self, MemoryError> {
        let item = inventory_item(&path, &file.metadata().map_err(store_io)?)?;
        Ok(Self { path, file, item })
    }

    pub(super) fn read(&mut self) -> Result<MemoryEntry, MemoryError> {
        read_entry_from_file(&self.path, &mut self.file)
    }

    pub(super) fn item(&self) -> &InventoryItem {
        &self.item
    }

    fn ensure_current(&self) -> Result<(), MemoryError> {
        self.path.ensure_same_file(&self.file)?;
        let current = inventory_item(&self.path, &self.file.metadata().map_err(store_io)?)?;
        if current == self.item {
            Ok(())
        } else {
            Err(unsafe_path())
        }
    }
}

pub(super) fn inventory_digest(items: &[InventoryItem]) -> Result<String, MemoryError> {
    let bytes = serde_json::to_vec(items).map_err(|_| store_error())?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

pub(super) fn max_index_bytes(item_count: usize) -> Result<u64, MemoryError> {
    let count = u64::try_from(item_count).map_err(|_| store_error())?;
    count
        .checked_mul(INDEX_ITEM_MAX_BYTES)
        .and_then(|items| items.checked_add(INDEX_HEADER_MAX_BYTES))
        .ok_or_else(store_error)
}

fn inventory_item(path: &ManagedPath, metadata: &Metadata) -> Result<InventoryItem, MemoryError> {
    Ok(InventoryItem {
        path: relative_string(path.relative())?,
        length: metadata.len(),
        modified_ns: modified_ns(metadata)?,
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

fn modified_ns(metadata: &Metadata) -> Result<i64, MemoryError> {
    metadata
        .mtime()
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(metadata.mtime_nsec()))
        .ok_or_else(store_error)
}

fn store_io(_: std::io::Error) -> MemoryError {
    store_error()
}

const fn store_error() -> MemoryError {
    MemoryError::new("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::new("unsafe_store_path", "store")
}
