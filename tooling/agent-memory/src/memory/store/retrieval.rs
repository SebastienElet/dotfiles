use super::canonicalization::kind_name;
use super::inventory::{read_entry_from_file, valid_memory_id};
use super::{Store, StorePhase};
use crate::memory::{MemoryEntry, MemoryError, SelectedMemory};
use std::os::unix::fs::MetadataExt;

impl Store {
    pub(crate) fn load_selected(
        &self,
        selected: &SelectedMemory,
    ) -> Result<MemoryEntry, MemoryError> {
        if !valid_memory_id(&selected.entry_id) {
            return Err(selection_stale());
        }
        let path = self.root.join(&selected.path)?;
        let mut file = path.open_read()?;
        ensure_snapshot(&file.metadata().map_err(|_| selection_stale())?, selected)?;
        let entry = read_entry_from_file(&path, &mut file)?;
        self.hit(StorePhase::AfterRetrievalEntryRead)?;
        path.ensure_same_file(&file)
            .map_err(|_| selection_stale())?;
        ensure_snapshot(&file.metadata().map_err(|_| selection_stale())?, selected)?;
        if entry.id().as_str() != selected.entry_id || kind_name(entry.kind()) != selected.kind {
            return Err(selection_stale());
        }
        Ok(entry)
    }
}

fn ensure_snapshot(
    metadata: &std::fs::Metadata,
    selected: &SelectedMemory,
) -> Result<(), MemoryError> {
    if metadata.len() == selected.length && modified_ns(metadata)? == selected.modified_ns {
        Ok(())
    } else {
        Err(selection_stale())
    }
}

fn modified_ns(metadata: &std::fs::Metadata) -> Result<i64, MemoryError> {
    metadata
        .mtime()
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(metadata.mtime_nsec()))
        .ok_or_else(selection_stale)
}

const fn selection_stale() -> MemoryError {
    MemoryError::new("selection_stale", "selection")
}
