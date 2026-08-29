use super::types::{StoreFailpoint, StorePhase};
use crate::memory::MemoryError;
use crate::memory::index::inventory::InventorySnapshot;
use crate::memory::path::ManagedPath;
use std::fs::File;
use std::io::{BufWriter, Write};

pub(crate) struct AtomicPublication<'a> {
    root: &'a ManagedPath,
    failpoint: Option<&'a StoreFailpoint>,
}

impl<'a> AtomicPublication<'a> {
    pub(super) fn new(root: &'a ManagedPath, failpoint: Option<&'a StoreFailpoint>) -> Self {
        Self { root, failpoint }
    }

    pub(super) fn initialize_file(
        &self,
        relative: &str,
        initial: &[u8],
        fail_mode_repair: bool,
    ) -> Result<(), MemoryError> {
        let path = self.root.join(relative)?;
        let mut file = path.ensure_file(fail_mode_repair)?;
        if file.metadata().map_err(store_io)?.len() == 0 {
            file.write_all(initial).map_err(store_io)?;
            file.sync_all().map_err(store_io)?;
        }
        Ok(())
    }

    pub(super) fn stage_yaml(
        &self,
        destination: &ManagedPath,
        bytes: &[u8],
    ) -> Result<StagedFile, MemoryError> {
        self.write_temporary(
            destination,
            bytes,
            StorePhase::BeforeYamlTemporaryCreate,
            StorePhase::BeforeYamlWrite,
            StorePhase::BeforeYamlFlush,
            StorePhase::BeforeYamlFsync,
        )
    }

    pub(crate) fn stage_index(&self, bytes: &[u8]) -> Result<StagedIndex, MemoryError> {
        let destination = self.root.join("index.json")?;
        let original = if destination.exists()? {
            Some(destination.open_read()?)
        } else {
            None
        };
        let staged = self.write_temporary(
            &destination,
            bytes,
            StorePhase::BeforeIndexTemporaryCreate,
            StorePhase::BeforeIndexWrite,
            StorePhase::BeforeIndexFlush,
            StorePhase::BeforeIndexFsync,
        )?;
        Ok(StagedIndex {
            staged,
            destination,
            original,
        })
    }

    pub(crate) fn publish_index(
        &self,
        index: StagedIndex,
        inventory: &InventorySnapshot,
    ) -> Result<(), MemoryError> {
        self.hit(StorePhase::BeforeIndexRename)?;
        inventory.ensure_current(self.root)?;
        index.ensure_anchored()?;
        index.publish()?;
        self.root.sync_directory()
    }

    pub(super) fn publish(
        &self,
        yaml: StagedFile,
        destination: &ManagedPath,
        index: StagedIndex,
        inventory: InventorySnapshot,
        replace: bool,
    ) -> Result<(), CommitFailure> {
        self.hit(StorePhase::BeforeYamlRename)
            .map_err(CommitFailure::BeforeYaml)?;
        yaml.ensure_anchored().map_err(CommitFailure::BeforeYaml)?;
        if replace {
            yaml.path.rename_to(destination)
        } else {
            yaml.path.rename_new_to(destination)
        }
        .map_err(CommitFailure::BeforeYaml)?;
        self.hit(StorePhase::AfterYamlRename)
            .map_err(|_| CommitFailure::AfterYaml)?;
        self.hit(StorePhase::BeforeYamlDirectoryFsync)
            .map_err(|_| CommitFailure::AfterYaml)?;
        destination
            .sync_parent_directory()
            .map_err(|_| CommitFailure::AfterYaml)?;
        self.hit(StorePhase::BeforeIndexRename)
            .map_err(|_| CommitFailure::AfterYaml)?;
        inventory
            .ensure_current(self.root)
            .map_err(|_| CommitFailure::AfterYaml)?;
        index
            .ensure_anchored()
            .map_err(|_| CommitFailure::AfterYaml)?;
        index.publish().map_err(|_| CommitFailure::AfterYaml)?;
        self.root
            .sync_directory()
            .map_err(|_| CommitFailure::AfterYaml)?;
        Ok(())
    }

    fn write_temporary(
        &self,
        destination: &ManagedPath,
        bytes: &[u8],
        create: StorePhase,
        write: StorePhase,
        flush: StorePhase,
        fsync: StorePhase,
    ) -> Result<StagedFile, MemoryError> {
        self.hit(create)?;
        let path = self.temporary_path(destination)?;
        let file = path.open_new()?;
        let mut writer = BufWriter::new(file);
        self.hit(write)?;
        writer.write_all(bytes).map_err(store_io)?;
        self.hit(flush)?;
        writer.flush().map_err(store_io)?;
        self.hit(fsync)?;
        writer.get_ref().sync_all().map_err(store_io)?;
        let file = writer.into_inner().map_err(|_| store_error())?;
        Ok(StagedFile { path, file })
    }

    fn temporary_path(&self, destination: &ManagedPath) -> Result<ManagedPath, MemoryError> {
        let relative = destination.relative();
        let parent = relative.parent().ok_or_else(store_error)?;
        let name = relative
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(store_error)?;
        let mut random = [0_u8; 16];
        getrandom::fill(&mut random).map_err(|_| store_error())?;
        let suffix = random
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        self.root.join(parent.join(format!(".{name}.tmp-{suffix}")))
    }

    fn hit(&self, phase: StorePhase) -> Result<(), MemoryError> {
        self.failpoint
            .map_or(Ok(()), |failpoint| failpoint.reach(phase))
    }
}

pub(crate) struct StagedFile {
    path: ManagedPath,
    file: File,
}

pub(crate) struct StagedIndex {
    staged: StagedFile,
    destination: ManagedPath,
    original: Option<File>,
}

impl StagedIndex {
    fn ensure_anchored(&self) -> Result<(), MemoryError> {
        self.staged.ensure_anchored()?;
        match &self.original {
            Some(original) => self.destination.ensure_same_file(original),
            None if self.destination.exists()? => Err(unsafe_path()),
            None => Ok(()),
        }
    }

    fn publish(self) -> Result<(), MemoryError> {
        self.staged.path.rename_to(&self.destination)
    }
}

impl StagedFile {
    pub(super) fn anchor(&self) -> Result<File, MemoryError> {
        self.file.try_clone().map_err(store_io)
    }

    fn ensure_anchored(&self) -> Result<(), MemoryError> {
        self.path.ensure_same_file(&self.file)
    }
}

#[derive(Debug)]
pub(super) enum CommitFailure {
    BeforeYaml(MemoryError),
    AfterYaml,
}

fn store_io(_: std::io::Error) -> MemoryError {
    store_error()
}

const fn store_error() -> MemoryError {
    MemoryError::new("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::new("unsafe_store_path", "store")
}
