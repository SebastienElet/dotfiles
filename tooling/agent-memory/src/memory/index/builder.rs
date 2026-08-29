use super::document::{IndexDiagnostics, IndexDocument, IndexRow, diagnostic_scope, index_bytes};
use super::inventory::{AnchoredEntry, InventorySnapshot};
use super::{IndexDiagnostic, entry_id};
use crate::memory::path::ManagedPath;
use crate::memory::store::document::StoredEntry;
use crate::memory::{MemoryError, Status, Store};
use std::fs::File;

pub(crate) struct PreparedIndex {
    pub(super) document: IndexDocument,
    pub(crate) bytes: Vec<u8>,
    pub(crate) inventory: InventorySnapshot,
}

pub(super) fn rebuild_index(
    store: &Store,
    mut inventory: InventorySnapshot,
) -> Result<PreparedIndex, MemoryError> {
    let (mut rows, mut diagnostics) = indexed_existing(store, &mut inventory, None)?;
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    diagnostics.sort();
    prepare(rows, diagnostics, inventory)
}

pub(super) fn prepare_with_candidate(
    store: &Store,
    destination: &ManagedPath,
    candidate: &StoredEntry,
    candidate_file: File,
    mut inventory: InventorySnapshot,
) -> Result<PreparedIndex, MemoryError> {
    let (mut rows, mut diagnostics) = indexed_existing(store, &mut inventory, Some(destination))?;
    inventory.replace(destination, candidate_file)?;
    let item = inventory.item_for(destination).ok_or_else(store_error)?;
    if candidate.status == "active" {
        rows.push(IndexRow::new(candidate, item));
    } else {
        diagnostics.push(&candidate.scope, diagnostic(&candidate.id, "status"));
    }
    rows.sort_by(|left, right| left.path.cmp(&right.path));
    diagnostics.sort();
    prepare(rows, diagnostics, inventory)
}

fn indexed_existing(
    store: &Store,
    inventory: &mut InventorySnapshot,
    excluded: Option<&ManagedPath>,
) -> Result<(Vec<IndexRow>, IndexDiagnostics), MemoryError> {
    let mut rows = Vec::new();
    let mut diagnostics = IndexDiagnostics::default();
    for anchored in inventory.entries_mut() {
        if excluded.is_some_and(|path| path.relative() == anchored.path.relative()) {
            continue;
        }
        index_entry(store, anchored, &mut rows, &mut diagnostics)?;
    }
    Ok((rows, diagnostics))
}

fn index_entry(
    store: &Store,
    anchored: &mut AnchoredEntry,
    rows: &mut Vec<IndexRow>,
    diagnostics: &mut IndexDiagnostics,
) -> Result<(), MemoryError> {
    store.before_index_entry_read()?;
    let result = anchored.read();
    store.after_index_entry_read()?;
    let scope = diagnostic_scope(anchored.path.relative())?;
    match result {
        Ok(entry) if entry.status() == Status::Active => {
            rows.push(IndexRow::new(
                &StoredEntry::from_entry(&entry),
                anchored.item(),
            ));
        }
        Ok(entry) => diagnostics.push(&scope, diagnostic(entry.id().as_str(), "status")),
        Err(error) => diagnostics.push(&scope, diagnostic(entry_id(&anchored.path)?, error.code())),
    }
    Ok(())
}

fn prepare(
    rows: Vec<IndexRow>,
    diagnostics: IndexDiagnostics,
    inventory: InventorySnapshot,
) -> Result<PreparedIndex, MemoryError> {
    let document = IndexDocument::with_inventory(rows, &inventory.items(), diagnostics)?;
    let bytes = index_bytes(&document)?;
    Ok(PreparedIndex {
        document,
        bytes,
        inventory,
    })
}

fn diagnostic(entry_id: &str, check: &str) -> IndexDiagnostic {
    IndexDiagnostic {
        entry_id: entry_id.to_owned(),
        check: check.to_owned(),
        effect: "omitted".to_owned(),
    }
}

const fn store_error() -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}
