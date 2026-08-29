use crate::memory::{MemoryEntry, MemoryError};
use std::sync::{Arc, Barrier};

#[derive(Clone, Debug)]
pub enum StoreFailpoint {
    BeforeModeRepair,
    AfterProjectDirectoryFsync,
    BeforeYamlTemporaryCreate,
    BeforeYamlWrite,
    BeforeYamlFlush,
    BeforeYamlFsync,
    BeforeYamlRename,
    AfterYamlRename,
    BeforeYamlDirectoryFsync,
    BeforeIndexTemporaryCreate,
    BeforeIndexWrite,
    BeforeIndexFlush,
    BeforeIndexFsync,
    BeforeIndexRename,
    PauseBeforeYamlRename(Arc<Barrier>),
    PauseBeforeIndexRename(Arc<Barrier>),
    PauseBeforeIndexEntryRead(Arc<Barrier>),
    PauseAfterIndexEntryRead(Arc<Barrier>),
    PauseAfterLockAcquire(Arc<Barrier>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum StorePhase {
    AfterLockAcquire,
    AfterProjectDirectoryFsync,
    BeforeYamlTemporaryCreate,
    BeforeYamlWrite,
    BeforeYamlFlush,
    BeforeYamlFsync,
    BeforeYamlRename,
    AfterYamlRename,
    BeforeYamlDirectoryFsync,
    BeforeIndexTemporaryCreate,
    BeforeIndexWrite,
    BeforeIndexFlush,
    BeforeIndexFsync,
    BeforeIndexRename,
    BeforeIndexEntryRead,
    AfterIndexEntryRead,
}

impl StoreFailpoint {
    pub(super) fn reach(&self, phase: StorePhase) -> Result<(), MemoryError> {
        match (self, phase) {
            (Self::PauseBeforeYamlRename(barrier), StorePhase::BeforeYamlRename)
            | (Self::PauseBeforeIndexRename(barrier), StorePhase::BeforeIndexRename) => {
                barrier.wait();
                barrier.wait();
                Ok(())
            }
            (Self::PauseBeforeIndexEntryRead(barrier), StorePhase::BeforeIndexEntryRead) => {
                barrier.wait();
                barrier.wait();
                Ok(())
            }
            (Self::PauseAfterIndexEntryRead(barrier), StorePhase::AfterIndexEntryRead) => {
                barrier.wait();
                barrier.wait();
                Ok(())
            }
            (Self::PauseAfterLockAcquire(barrier), StorePhase::AfterLockAcquire) => {
                barrier.wait();
                barrier.wait();
                Ok(())
            }
            _ if self.matches(phase) => Err(MemoryError::new("store_unavailable", "store")),
            _ => Ok(()),
        }
    }

    fn matches(&self, phase: StorePhase) -> bool {
        matches!(
            (self, phase),
            (
                Self::AfterProjectDirectoryFsync,
                StorePhase::AfterProjectDirectoryFsync
            ) | (
                Self::BeforeYamlTemporaryCreate,
                StorePhase::BeforeYamlTemporaryCreate
            ) | (Self::BeforeYamlWrite, StorePhase::BeforeYamlWrite)
                | (Self::BeforeYamlFlush, StorePhase::BeforeYamlFlush)
                | (Self::BeforeYamlFsync, StorePhase::BeforeYamlFsync)
                | (Self::BeforeYamlRename, StorePhase::BeforeYamlRename)
                | (Self::AfterYamlRename, StorePhase::AfterYamlRename)
                | (
                    Self::BeforeYamlDirectoryFsync,
                    StorePhase::BeforeYamlDirectoryFsync
                )
                | (
                    Self::BeforeIndexTemporaryCreate,
                    StorePhase::BeforeIndexTemporaryCreate
                )
                | (Self::BeforeIndexWrite, StorePhase::BeforeIndexWrite)
                | (Self::BeforeIndexFlush, StorePhase::BeforeIndexFlush)
                | (Self::BeforeIndexFsync, StorePhase::BeforeIndexFsync)
                | (Self::BeforeIndexRename, StorePhase::BeforeIndexRename)
        )
    }
}

#[derive(Debug)]
pub struct StoreCommit {
    pub(super) index_rebuild_required: bool,
}

impl StoreCommit {
    pub fn index_rebuild_required(&self) -> bool {
        self.index_rebuild_required
    }
}

#[derive(Debug)]
pub struct StoreListing {
    pub(super) entries: Vec<MemoryEntry>,
    pub(super) index_rebuild_required: bool,
}

impl StoreListing {
    pub fn entries(&self) -> &[MemoryEntry] {
        &self.entries
    }

    pub fn index_rebuild_required(&self) -> bool {
        self.index_rebuild_required
    }
}
