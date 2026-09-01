mod builder;
mod document;
pub(crate) mod inventory;

use self::builder::{PreparedIndex, prepare_with_candidate, rebuild_index};
pub(crate) use self::document::IndexRow;
use self::document::{IndexDocument, index_bytes};
use self::inventory::{InventorySnapshot, max_index_bytes};
use super::path::ManagedPath;
use super::store::document::StoredEntry;
use super::store::inventory::valid_memory_id;
use super::{MemoryError, Store};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;

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
        let inventory = InventorySnapshot::capture(store.root())?;
        if let Some(document) = load_current(store.root(), &inventory)? {
            return Ok(Self::loaded(document, false));
        }
        let _lock = store.acquire_lock()?;
        let inventory = InventorySnapshot::capture(store.root())?;
        if let Some(document) = load_current(store.root(), &inventory)? {
            return Ok(Self::loaded(document, false));
        }
        let prepared = rebuild_index(store, inventory)?;
        publish_rebuild(store, &prepared)?;
        Ok(Self::loaded(prepared.document, true))
    }

    pub(super) fn entries(&self) -> &[IndexRow] {
        &self.document.entries
    }

    pub(super) fn diagnostics_for(
        &self,
        project_key: &str,
        include_user: bool,
    ) -> Vec<IndexDiagnostic> {
        self.document
            .diagnostics
            .for_scope(project_key, include_user)
    }

    fn loaded(document: IndexDocument, rebuilt: bool) -> IndexLoad {
        let diagnostics = document.diagnostics.all();
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

pub(super) fn prepared_index(
    store: &Store,
    destination: &ManagedPath,
    candidate: &StoredEntry,
    candidate_file: File,
) -> Result<PreparedIndex, MemoryError> {
    prepare_with_candidate(
        store,
        destination,
        candidate,
        candidate_file,
        InventorySnapshot::capture(store.root())?,
    )
}

pub(super) fn index_rebuild_required(root: &ManagedPath) -> Result<bool, MemoryError> {
    let inventory = InventorySnapshot::capture(root)?;
    Ok(load_current(root, &inventory)?.is_none())
}

fn load_current(
    root: &ManagedPath,
    inventory: &InventorySnapshot,
) -> Result<Option<IndexDocument>, MemoryError> {
    let Some(document) = load_fresh(root, inventory)? else {
        return Ok(None);
    };
    Ok(inventory.matches_current(root)?.then_some(document))
}

fn load_fresh(
    root: &ManagedPath,
    inventory: &InventorySnapshot,
) -> Result<Option<IndexDocument>, MemoryError> {
    let items = inventory.items();
    let Some(bytes) = read_index(root, max_index_bytes(items.len())?)? else {
        return Ok(None);
    };
    let Ok(document) = serde_json::from_slice::<IndexDocument>(&bytes) else {
        return Ok(None);
    };
    Ok(document.valid_for(&items).then_some(document))
}

fn read_index(root: &ManagedPath, maximum: u64) -> Result<Option<Vec<u8>>, MemoryError> {
    let path = root.join("index.json")?;
    if !path.exists()? {
        return Ok(None);
    }
    let mut file = path.open_read_only()?;
    let metadata = file.metadata().map_err(store_io)?;
    if metadata.len() > maximum {
        return Ok(None);
    }
    let mut bytes = Vec::with_capacity(usize::try_from(metadata.len()).map_err(|_| store_error())?);
    Read::by_ref(&mut file)
        .take(maximum.checked_add(1).ok_or_else(store_error)?)
        .read_to_end(&mut bytes)
        .map_err(store_io)?;
    Ok(((bytes.len() as u64) <= maximum).then_some(bytes))
}

fn publish_rebuild(store: &Store, prepared: &PreparedIndex) -> Result<(), MemoryError> {
    let staged = store.publication().stage_index(&prepared.bytes)?;
    store
        .publication()
        .publish_index(staged, &prepared.inventory)
}

pub(super) fn entry_id(path: &ManagedPath) -> Result<&str, MemoryError> {
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

fn store_io(_: std::io::Error) -> MemoryError {
    store_error()
}

const fn store_error() -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::unavailable("unsafe_store_path", "store")
}
