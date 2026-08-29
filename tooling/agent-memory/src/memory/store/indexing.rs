use super::Store;
use super::document::{StoredEntry, yaml_bytes};
use super::publication::CommitFailure;
use super::types::StorePhase;
use crate::memory::MemoryError;
use crate::memory::index::prepared_index;
use crate::memory::path::ManagedPath;

impl Store {
    pub(super) fn commit_entry<F>(
        &self,
        destination: &ManagedPath,
        entry: &StoredEntry,
        replace: bool,
        before_publish: F,
    ) -> Result<(), CommitFailure>
    where
        F: FnOnce() -> Result<(), MemoryError>,
    {
        let publication = self.publication();
        let yaml = yaml_bytes(entry).map_err(CommitFailure::BeforeYaml)?;
        let staged_yaml = publication
            .stage_yaml(destination, &yaml)
            .map_err(CommitFailure::BeforeYaml)?;
        let index = prepared_index(
            self,
            destination,
            entry,
            staged_yaml.anchor().map_err(CommitFailure::BeforeYaml)?,
        )
        .map_err(CommitFailure::BeforeYaml)?;
        let staged_index = publication
            .stage_index(&index.bytes)
            .map_err(CommitFailure::BeforeYaml)?;
        publication.publish(
            staged_yaml,
            destination,
            staged_index,
            index.inventory,
            replace,
            before_publish,
        )
    }

    pub(crate) fn before_index_entry_read(&self) -> Result<(), MemoryError> {
        self.hit(StorePhase::BeforeIndexEntryRead)
    }

    pub(crate) fn after_index_entry_read(&self) -> Result<(), MemoryError> {
        self.hit(StorePhase::AfterIndexEntryRead)
    }
}
