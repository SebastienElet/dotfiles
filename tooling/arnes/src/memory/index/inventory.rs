use super::document::relative_string;
use crate::memory::MemoryError;
use crate::memory::path::ManagedPath;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::os::unix::fs::MetadataExt;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub(super) struct InventoryItem {
    pub(super) path: String,
    pub(super) length: u64,
    pub(super) modified_ns: i64,
    #[serde(skip)]
    pub(super) device: u64,
    #[serde(skip)]
    pub(super) inode: u64,
}

pub(super) fn inventory(paths: &[ManagedPath]) -> Result<Vec<InventoryItem>, MemoryError> {
    let mut items = paths
        .iter()
        .map(|path| {
            let metadata = path.open_read()?.metadata().map_err(store_io)?;
            Ok(InventoryItem {
                path: relative_string(path.relative())?,
                length: metadata.len(),
                modified_ns: modified_ns(&metadata)?,
                device: metadata.dev(),
                inode: metadata.ino(),
            })
        })
        .collect::<Result<Vec<_>, MemoryError>>()?;
    items.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(items)
}

pub(super) fn prepared_inventory(
    destination: &ManagedPath,
    candidate_metadata: &std::fs::Metadata,
    paths: &[ManagedPath],
) -> Result<Vec<InventoryItem>, MemoryError> {
    let mut items = inventory(paths)?;
    let candidate = InventoryItem {
        path: relative_string(destination.relative())?,
        length: candidate_metadata.len(),
        modified_ns: modified_ns(candidate_metadata)?,
        device: candidate_metadata.dev(),
        inode: candidate_metadata.ino(),
    };
    if let Some(existing) = items.iter_mut().find(|item| item.path == candidate.path) {
        *existing = candidate;
    } else {
        items.push(candidate);
    }
    items.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(items)
}

pub(super) fn inventory_digest(items: &[InventoryItem]) -> Result<String, MemoryError> {
    let bytes = serde_json::to_vec(items).map_err(|_| store_error())?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn modified_ns(metadata: &std::fs::Metadata) -> Result<i64, MemoryError> {
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
