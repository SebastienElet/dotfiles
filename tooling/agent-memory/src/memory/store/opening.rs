use super::inventory::{repair_entry_modes, validate_entry_modes};
use super::{Store, StoreFailpoint};
use crate::memory::MemoryError;
use crate::memory::index::empty_index_bytes;
use crate::memory::path::{
    DirectoryAccess, ManagedPath, MemoryRoot, open_existing_root, open_root,
};

impl Store {
    pub fn open(root: MemoryRoot) -> Result<Self, MemoryError> {
        Self::open_internal(root, None)
    }

    pub fn open_with_failpoint(
        root: MemoryRoot,
        failpoint: StoreFailpoint,
    ) -> Result<Self, MemoryError> {
        Self::open_internal(root, Some(failpoint))
    }

    fn open_internal(
        root: MemoryRoot,
        failpoint: Option<StoreFailpoint>,
    ) -> Result<Self, MemoryError> {
        let fail_mode_repair = matches!(failpoint.as_ref(), Some(StoreFailpoint::BeforeModeRepair));
        let root = ManagedPath::root(open_root(&root, fail_mode_repair)?, DirectoryAccess::Repair);
        for relative in ["entries", "entries/user", "entries/project"] {
            root.join(relative)?.ensure_directory(fail_mode_repair)?;
        }
        repair_entry_modes(&root)?;
        root.join(".lock")?.ensure_file(fail_mode_repair)?;
        let store = Self { root, failpoint };
        let publication = store.publication();
        publication.initialize_file("index.json", &empty_index_bytes()?, fail_mode_repair)?;
        publication.initialize_file(
            "oracle-cache.json",
            b"{\n  \"schema_version\": 1,\n  \"entries\": []\n}\n",
            fail_mode_repair,
        )?;
        Ok(store)
    }

    pub fn open_read_only(root: MemoryRoot) -> Result<Option<Self>, MemoryError> {
        Self::open_existing(root, false)
    }

    pub fn open_for_retrieval(root: MemoryRoot) -> Result<Option<Self>, MemoryError> {
        Self::open_existing(root, true)
    }

    fn open_existing(
        root: MemoryRoot,
        initialize_derived: bool,
    ) -> Result<Option<Self>, MemoryError> {
        let Some(root) = open_existing_root(&root)? else {
            return Ok(None);
        };
        let root = ManagedPath::root(root, DirectoryAccess::Validate);
        for relative in ["entries", "entries/user", "entries/project"] {
            root.join(relative)?.validate_directory()?;
        }
        validate_entry_modes(&root)?;
        let store = Self {
            root,
            failpoint: None,
        };
        if initialize_derived {
            store.initialize_derived()?;
        }
        Ok(Some(store))
    }

    fn initialize_derived(&self) -> Result<(), MemoryError> {
        self.root.join(".lock")?.ensure_file(false)?;
        let publication = self.publication();
        publication.initialize_file("index.json", &empty_index_bytes()?, false)?;
        publication.initialize_file(
            "oracle-cache.json",
            b"{\n  \"schema_version\": 1,\n  \"entries\": []\n}\n",
            false,
        )
    }
}
