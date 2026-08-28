use crate::memory::MemoryEntry;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StoreFailpoint {
    BeforeModeRepair,
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
