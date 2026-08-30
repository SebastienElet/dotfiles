pub(super) mod canonicalization;
pub(super) mod document;
mod indexing;
pub(super) mod inventory;
mod opening;
pub(super) mod publication;
mod retrieval;
mod staging;
pub(super) mod types;

use self::document::{StoredEntry, StoredScope};
use self::inventory::{entry_paths, read_entry, valid_memory_id};
use self::publication::{AtomicPublication, CommitFailure};
use self::types::StorePhase;
pub use self::types::{StoreCommit, StoreFailpoint, StoreListing};
use super::index::index_rebuild_required;
use super::lock::GlobalLock;
use super::path::ManagedPath;
use super::{
    AdmissionResult, MemoryEntry, MemoryError, MemoryId, ProjectScope, ResolvedDraft,
    SourceContext, Status, UtcTimestamp,
};

#[derive(Debug)]
pub struct Store {
    root: ManagedPath,
    failpoint: Option<StoreFailpoint>,
}

impl Store {
    pub fn admit(
        &self,
        resolved: ResolvedDraft,
        project: Option<&ProjectScope>,
        timestamp: &UtcTimestamp,
        sources: &SourceContext<'_>,
    ) -> AdmissionResult {
        let candidate = match StoredEntry::from_resolved(&resolved, project, timestamp) {
            Ok(candidate) => candidate,
            Err(error) => return AdmissionResult::Rejected { error },
        };
        let id = MemoryId::from_validated(candidate.id.clone());
        let destination = match self.entry_path(&candidate) {
            Ok(destination) => destination,
            Err(error) => return AdmissionResult::Rejected { error },
        };
        let _lock = match self.acquire_lock() {
            Ok(lock) => lock,
            Err(error) => return AdmissionResult::Conflict { id, error },
        };
        if let Err(error) = resolved.recheck_sources(sources) {
            return AdmissionResult::Conflict { id, error };
        }
        if let Err(error) = self.ensure_entry_parent(&candidate) {
            return AdmissionResult::Rejected { error };
        }
        match self.existing_admission(&destination, &candidate) {
            Ok(Some(true)) => return AdmissionResult::Duplicate { id },
            Ok(Some(false)) | Err(_) => {
                return AdmissionResult::Conflict {
                    id,
                    error: MemoryError::conflict("entry_conflict", "id"),
                };
            }
            Ok(None) => {}
        }
        let commit = self.commit_entry(&destination, &candidate, false, || {
            resolved.recheck_sources(sources)
        });
        admission_commit(id, commit)
    }

    pub fn replace_active(&self, entry: &MemoryEntry) -> Result<StoreCommit, MemoryError> {
        if entry.status() == Status::Active {
            return Err(MemoryError::new("entry_not_terminal", "status"));
        }
        let candidate = StoredEntry::from_entry(entry);
        let destination = self.entry_path(&candidate)?;
        let _lock = self.acquire_lock()?;
        let current = read_entry(&destination)?;
        if current.status() != Status::Active {
            return Err(MemoryError::conflict("entry_not_active", "status"));
        }
        if StoredEntry::from_entry(&current).immutable_value()? != candidate.immutable_value()? {
            return Err(MemoryError::conflict("entry_conflict", "id"));
        }
        match self.commit_entry(&destination, &candidate, true, || Ok(())) {
            Ok(()) => Ok(StoreCommit {
                index_rebuild_required: false,
            }),
            Err(CommitFailure::BeforeYaml(error)) => Err(error),
            Err(CommitFailure::AfterYaml) => Ok(StoreCommit {
                index_rebuild_required: true,
            }),
        }
    }

    pub fn load(&self, id: &str) -> Result<Option<MemoryEntry>, MemoryError> {
        if !valid_memory_id(id) {
            return Err(MemoryError::new("invalid_memory_id", "id"));
        }
        let mut matches = self
            .list()?
            .entries
            .into_iter()
            .filter(|entry| entry.id().as_str() == id);
        let found = matches.next();
        if matches.next().is_some() {
            return Err(MemoryError::conflict("entry_conflict", "id"));
        }
        Ok(found)
    }

    pub fn list(&self) -> Result<StoreListing, MemoryError> {
        let paths = entry_paths(&self.root)?;
        let entries = paths
            .iter()
            .map(read_entry)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(StoreListing {
            entries,
            index_rebuild_required: index_rebuild_required(&self.root)?,
        })
    }

    fn existing_admission(
        &self,
        path: &ManagedPath,
        candidate: &StoredEntry,
    ) -> Result<Option<bool>, MemoryError> {
        if !path.exists()? {
            return Ok(None);
        }
        let existing = StoredEntry::from_entry(&read_entry(path)?);
        Ok(Some(
            existing.canonical_value()? == candidate.canonical_value()?,
        ))
    }

    pub(super) fn acquire_lock(&self) -> Result<GlobalLock, MemoryError> {
        let path = self.root.join(".lock")?;
        let lock = GlobalLock::acquire(&self.root, &path)?;
        self.hit(StorePhase::AfterLockAcquire)?;
        lock.ensure_anchored(&path)?;
        Ok(lock)
    }

    pub(super) fn publication(&self) -> AtomicPublication<'_> {
        AtomicPublication::new(&self.root, self.failpoint.as_ref())
    }

    pub(super) fn root(&self) -> &ManagedPath {
        &self.root
    }

    fn entry_path(&self, entry: &StoredEntry) -> Result<ManagedPath, MemoryError> {
        match &entry.scope {
            StoredScope::User => self.root.join(format!("entries/user/{}.yaml", entry.id)),
            StoredScope::Project { key } => self
                .root
                .join(format!("entries/project/{key}/{}.yaml", entry.id)),
        }
    }

    fn ensure_entry_parent(&self, entry: &StoredEntry) -> Result<(), MemoryError> {
        if let StoredScope::Project { key } = &entry.scope {
            let directory = self.root.join(format!("entries/project/{key}"))?;
            directory.ensure_directory(false)?;
            directory.sync_parent_directory()?;
            self.hit(StorePhase::AfterProjectDirectoryFsync)?;
        }
        Ok(())
    }

    pub(super) fn hit(&self, phase: StorePhase) -> Result<(), MemoryError> {
        self.failpoint
            .as_ref()
            .map_or(Ok(()), |failpoint| failpoint.reach(phase))
    }
}

fn admission_failure(id: MemoryId, error: MemoryError) -> AdmissionResult {
    if error.class() == super::MemoryErrorClass::Conflict {
        AdmissionResult::Conflict { id, error }
    } else {
        AdmissionResult::Rejected { error }
    }
}

fn admission_commit(id: MemoryId, result: Result<(), CommitFailure>) -> AdmissionResult {
    match result {
        Ok(()) => AdmissionResult::Stored {
            id,
            index_rebuild_required: false,
        },
        Err(CommitFailure::BeforeYaml(error)) => admission_failure(id, error),
        Err(CommitFailure::AfterYaml) => AdmissionResult::Stored {
            id,
            index_rebuild_required: true,
        },
    }
}
