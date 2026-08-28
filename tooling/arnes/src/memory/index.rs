mod document;
mod inventory;

pub(crate) use self::document::IndexRow;
use self::document::{IndexDocument, index_bytes};
use self::inventory::{InventoryItem, inventory, prepared_inventory};
use super::path::ManagedPath;
use super::store::document::StoredEntry;
use super::store::inventory::{entry_paths, read_entry, valid_memory_id};
use super::{MemoryError, Status, Store};
use serde::{Deserialize, Serialize};
use std::fs::Metadata;
use std::io::Read;

const MAX_INDEX_READ_BYTES: u64 = 1024 * 1024;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct IndexDiagnostic {
    pub entry_id: String,
    pub check: String,
    pub effect: String,
}

#[derive(Debug)]
pub struct IndexLoad {
    pub index: Index,
    pub rebuilt: bool,
    pub diagnostics: Vec<IndexDiagnostic>,
}

#[derive(Debug)]
pub struct Index {
    document: IndexDocument,
}

impl Index {
    pub fn load_or_rebuild(store: &Store) -> Result<IndexLoad, MemoryError> {
        let paths = entry_paths(store.root())?;
        let current_inventory = inventory(&paths)?;
        if let Some(document) = load_fresh(store.root(), &current_inventory)? {
            return Ok(Self::loaded(document, false));
        }
        let _lock = store.acquire_lock()?;
        let paths = entry_paths(store.root())?;
        let current_inventory = inventory(&paths)?;
        if let Some(document) = load_fresh(store.root(), &current_inventory)? {
            return Ok(Self::loaded(document, false));
        }
        let document = rebuild_document(store, &paths, &current_inventory)?;
        if inventory(&paths)? != current_inventory {
            return Err(unsafe_path());
        }
        let bytes = index_bytes(&document)?;
        let staged = store.publication().stage_index(&bytes)?;
        store.publication().publish_index(staged)?;
        Ok(Self::loaded(document, true))
    }

    pub(super) fn entries(&self) -> &[IndexRow] {
        &self.document.entries
    }

    pub(super) fn diagnostics(&self) -> &[IndexDiagnostic] {
        &self.document.diagnostics
    }

    fn loaded(document: IndexDocument, rebuilt: bool) -> IndexLoad {
        let diagnostics = document.diagnostics.clone();
        IndexLoad {
            index: Self { document },
            rebuilt,
            diagnostics,
        }
    }
}

pub(super) fn empty_index_bytes() -> Result<Vec<u8>, MemoryError> {
    index_bytes(&IndexDocument::empty()?)
}

pub(super) fn prepared_index_bytes(
    destination: &ManagedPath,
    candidate: &StoredEntry,
    candidate_metadata: &Metadata,
    paths: &[ManagedPath],
) -> Result<Vec<u8>, MemoryError> {
    let mut rows = Vec::new();
    let mut diagnostics = Vec::new();
    for path in paths
        .iter()
        .filter(|path| path.relative() != destination.relative())
    {
        let (row, row_diagnostic) = active_row(path)?;
        rows.extend(row);
        diagnostics.extend(row_diagnostic);
    }
    if candidate.status == "active" {
        rows.push(IndexRow::new(candidate, destination, candidate_metadata)?);
    } else {
        diagnostics.push(diagnostic(&candidate.id, "status"));
    }
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    sort_diagnostics(&mut diagnostics);
    let prepared = prepared_inventory(destination, candidate_metadata, paths)?;
    index_bytes(&IndexDocument::with_inventory(
        rows,
        &prepared,
        diagnostics,
    )?)
}

pub(super) fn index_rebuild_required(
    root: &ManagedPath,
    paths: &[ManagedPath],
) -> Result<bool, MemoryError> {
    let inventory = inventory(paths)?;
    Ok(load_fresh(root, &inventory)?.is_none())
}

fn active_row(
    path: &ManagedPath,
) -> Result<(Option<IndexRow>, Option<IndexDiagnostic>), MemoryError> {
    let entry = read_entry(path)?;
    if entry.status() != Status::Active {
        return Ok((None, Some(diagnostic(entry.id().as_str(), "status"))));
    }
    let stored = StoredEntry::from_entry(&entry);
    let metadata = path.open_read()?.metadata().map_err(store_io)?;
    Ok((Some(IndexRow::new(&stored, path, &metadata)?), None))
}

fn rebuild_document(
    store: &Store,
    paths: &[ManagedPath],
    inventory: &[InventoryItem],
) -> Result<IndexDocument, MemoryError> {
    let mut rows = Vec::new();
    let mut diagnostics = Vec::new();
    for path in paths {
        store.before_index_entry_read()?;
        match read_entry(path) {
            Ok(entry) if entry.status() == Status::Active => {
                let stored = StoredEntry::from_entry(&entry);
                let metadata = path.open_read()?.metadata().map_err(store_io)?;
                rows.push(IndexRow::new(&stored, path, &metadata)?);
            }
            Ok(entry) => diagnostics.push(diagnostic(entry.id().as_str(), "status")),
            Err(error) => diagnostics.push(diagnostic(entry_id(path)?, error.code())),
        }
    }
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    sort_diagnostics(&mut diagnostics);
    IndexDocument::with_inventory(rows, inventory, diagnostics)
}

fn load_fresh(
    root: &ManagedPath,
    inventory: &[InventoryItem],
) -> Result<Option<IndexDocument>, MemoryError> {
    let Some(bytes) = read_index(root)? else {
        return Ok(None);
    };
    let Ok(document) = serde_json::from_slice::<IndexDocument>(&bytes) else {
        return Ok(None);
    };
    Ok(document.valid_for(inventory).then_some(document))
}

fn read_index(root: &ManagedPath) -> Result<Option<Vec<u8>>, MemoryError> {
    let path = root.join("index.json")?;
    if !path.exists()? {
        return Ok(None);
    }
    let mut file = path.open_read()?;
    let metadata = file.metadata().map_err(store_io)?;
    if metadata.len() > MAX_INDEX_READ_BYTES {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    Read::by_ref(&mut file)
        .take(MAX_INDEX_READ_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(store_io)?;
    Ok((bytes.len() as u64 <= MAX_INDEX_READ_BYTES).then_some(bytes))
}

fn entry_id(path: &ManagedPath) -> Result<&str, MemoryError> {
    let id = path
        .relative()
        .file_stem()
        .and_then(|value| value.to_str())
        .ok_or_else(unsafe_path)?;
    if valid_memory_id(id) {
        Ok(id)
    } else {
        Err(unsafe_path())
    }
}

fn diagnostic(entry_id: &str, check: &str) -> IndexDiagnostic {
    IndexDiagnostic {
        entry_id: entry_id.to_owned(),
        check: check.to_owned(),
        effect: "omitted".to_owned(),
    }
}

fn sort_diagnostics(diagnostics: &mut [IndexDiagnostic]) {
    diagnostics.sort_by(|left, right| {
        (&left.entry_id, &left.check, &left.effect).cmp(&(
            &right.entry_id,
            &right.check,
            &right.effect,
        ))
    });
}

fn store_io(_: std::io::Error) -> MemoryError {
    MemoryError::new("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::new("unsafe_store_path", "store")
}
