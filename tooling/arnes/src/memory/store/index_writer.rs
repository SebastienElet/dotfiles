use super::document::{StoredEntry, StoredScope};
use super::inventory::read_entry;
use crate::memory::MemoryError;
use crate::memory::path::ManagedPath;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::Metadata;
use std::io::Read;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

const MAX_INDEX_READ_BYTES: u64 = 1024 * 1024;

#[derive(Debug, Deserialize, Serialize)]
pub(super) struct IndexDocument {
    schema_version: u8,
    inventory_digest: String,
    entries: Vec<IndexRow>,
}

impl IndexDocument {
    fn new(entries: Vec<IndexRow>) -> Result<Self, MemoryError> {
        let inventory = entries
            .iter()
            .map(|row| InventoryItem {
                path: row.path.clone(),
                length: row.length,
                modified_ns: row.modified_ns,
            })
            .collect::<Vec<_>>();
        Ok(Self {
            schema_version: 1,
            inventory_digest: inventory_digest(&inventory)?,
            entries,
        })
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct IndexRow {
    id: String,
    kind: String,
    status: String,
    scope: StoredScope,
    retrieval_terms: Vec<String>,
    summary: String,
    path: String,
    length: u64,
    modified_ns: i64,
}

impl IndexRow {
    fn new(
        entry: &StoredEntry,
        path: &ManagedPath,
        metadata: &Metadata,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            id: entry.id.clone(),
            kind: entry.kind.clone(),
            status: entry.status.clone(),
            scope: entry.scope.clone(),
            retrieval_terms: entry.retrieval_terms.clone(),
            summary: entry.proof.summary.chars().take(160).collect(),
            path: relative_string(path.relative())?,
            length: metadata.len(),
            modified_ns: modified_ns(metadata)?,
        })
    }
}

#[derive(Debug, Serialize)]
struct InventoryItem {
    path: String,
    length: u64,
    modified_ns: i64,
}

pub(super) fn empty_index_bytes() -> Result<Vec<u8>, MemoryError> {
    json_bytes(&IndexDocument::new(Vec::new())?)
}

pub(super) fn prepared_index_bytes(
    destination: &ManagedPath,
    candidate: &StoredEntry,
    candidate_metadata: &Metadata,
    paths: &[ManagedPath],
) -> Result<Vec<u8>, MemoryError> {
    let mut rows = Vec::new();
    for path in paths {
        if path.relative() != destination.relative() {
            let entry = StoredEntry::from_entry(&read_entry(path)?);
            let metadata = path.open_read()?.metadata().map_err(store_io)?;
            rows.push(IndexRow::new(&entry, path, &metadata)?);
        }
    }
    rows.push(IndexRow::new(candidate, destination, candidate_metadata)?);
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    json_bytes(&IndexDocument::new(rows)?)
}

pub(super) fn index_rebuild_required(
    root: &ManagedPath,
    paths: &[ManagedPath],
) -> Result<bool, MemoryError> {
    let digest = inventory_digest(&inventory(paths)?)?;
    let Some(bytes) = read_index(root)? else {
        return Ok(true);
    };
    let index = serde_json::from_slice::<IndexDocument>(&bytes);
    Ok(match index {
        Ok(index) => index.inventory_digest != digest,
        Err(_) => true,
    })
}

fn read_index(root: &ManagedPath) -> Result<Option<Vec<u8>>, MemoryError> {
    let mut file = root.join("index.json")?.open_read()?;
    let metadata = file.metadata().map_err(store_io)?;
    if metadata.len() > MAX_INDEX_READ_BYTES {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(&mut file)
        .take(MAX_INDEX_READ_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(store_io)?;
    if bytes.len() as u64 > MAX_INDEX_READ_BYTES {
        return Ok(None);
    }
    Ok(Some(bytes))
}

fn inventory(paths: &[ManagedPath]) -> Result<Vec<InventoryItem>, MemoryError> {
    let mut inventory = paths
        .iter()
        .map(|path| {
            let metadata = path.open_read()?.metadata().map_err(store_io)?;
            Ok(InventoryItem {
                path: relative_string(path.relative())?,
                length: metadata.len(),
                modified_ns: modified_ns(&metadata)?,
            })
        })
        .collect::<Result<Vec<_>, MemoryError>>()?;
    inventory.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(inventory)
}

fn inventory_digest(inventory: &[InventoryItem]) -> Result<String, MemoryError> {
    let bytes = serde_json::to_vec(inventory).map_err(|_| store_error())?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, MemoryError> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|_| store_error())?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn relative_string(path: &Path) -> Result<String, MemoryError> {
    path.to_str().map(str::to_owned).ok_or_else(unsafe_path)
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
