use super::Store;
use super::document::{StoredEntry, yaml_bytes};
use super::publication::CommitFailure;
use super::staging::StagedFile;
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
        let anchor = match staged_yaml.anchor() {
            Ok(anchor) => anchor,
            Err(error) => return Err(cleanup_yaml(staged_yaml, error)),
        };
        let index = match prepared_index(self, destination, entry, anchor) {
            Ok(index) => index,
            Err(error) => return Err(cleanup_yaml(staged_yaml, error)),
        };
        let staged_index = match publication.stage_index(&index.bytes) {
            Ok(index) => index,
            Err(error) => return Err(cleanup_yaml(staged_yaml, error)),
        };
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

fn cleanup_yaml(yaml: StagedFile, error: MemoryError) -> CommitFailure {
    CommitFailure::BeforeYaml(yaml.discard().err().unwrap_or(error))
}
