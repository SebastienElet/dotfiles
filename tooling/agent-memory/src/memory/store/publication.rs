use super::staging::{StagedFile, StagedIndex, cleanup_temporary};
use super::types::{StoreFailpoint, StorePhase};
use crate::memory::MemoryError;
use crate::memory::index::inventory::InventorySnapshot;
use crate::memory::path::ManagedPath;
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
        self.stage_derived(
            "index.json",
            bytes,
            StorePhase::BeforeIndexTemporaryCreate,
            StorePhase::BeforeIndexWrite,
            StorePhase::BeforeIndexFlush,
            StorePhase::BeforeIndexFsync,
        )
    }

    pub(crate) fn stage_cache(&self, bytes: &[u8]) -> Result<StagedIndex, MemoryError> {
        self.stage_derived(
            "oracle-cache.json",
            bytes,
            StorePhase::BeforeCacheTemporaryCreate,
            StorePhase::BeforeCacheWrite,
            StorePhase::BeforeCacheFlush,
            StorePhase::BeforeCacheFsync,
        )
    }

    fn stage_derived(
        &self,
        relative: &str,
        bytes: &[u8],
        create: StorePhase,
        write: StorePhase,
        flush: StorePhase,
        fsync: StorePhase,
    ) -> Result<StagedIndex, MemoryError> {
        let destination = self.root.join(relative)?;
        let original = if destination.exists()? {
            Some(destination.open_read()?)
        } else {
            None
        };
        let staged = self.write_temporary(&destination, bytes, create, write, flush, fsync)?;
        Ok(StagedIndex::new(staged, destination, original))
    }

    pub(crate) fn publish_cache(&self, cache: StagedIndex) -> Result<(), MemoryError> {
        if let Err(error) = self
            .hit(StorePhase::BeforeCacheRename)
            .and_then(|()| cache.ensure_anchored())
        {
            return Err(cleanup_derived(cache, error));
        }
        cache.publish()?;
        self.root.sync_directory()
    }

    pub(crate) fn publish_index(
        &self,
        index: StagedIndex,
        inventory: &InventorySnapshot,
    ) -> Result<(), MemoryError> {
        if let Err(error) = self
            .hit(StorePhase::BeforeIndexRename)
            .and_then(|()| inventory.ensure_current(self.root))
            .and_then(|()| index.ensure_anchored())
        {
            return Err(cleanup_derived(index, error));
        }
        index.publish()?;
        self.root.sync_directory()
    }

    pub(super) fn publish<F>(
        &self,
        yaml: StagedFile,
        destination: &ManagedPath,
        index: StagedIndex,
        inventory: InventorySnapshot,
        replace: bool,
        before_publish: F,
    ) -> Result<(), CommitFailure>
    where
        F: FnOnce() -> Result<(), MemoryError>,
    {
        let before_rename = self
            .hit(StorePhase::BeforeYamlRename)
            .and_then(|()| before_publish())
            .and_then(|()| yaml.ensure_anchored());
        if let Err(error) = before_rename {
            return Err(cleanup_before_yaml(yaml, index, error));
        }
        let rename = yaml.rename_to(destination, replace);
        if let Err(error) = rename {
            return Err(cleanup_before_yaml(yaml, index, error));
        }
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
        let write_result = self
            .hit(write)
            .and_then(|()| writer.write_all(bytes).map_err(store_io))
            .and_then(|()| self.hit(flush))
            .and_then(|()| writer.flush().map_err(store_io))
            .and_then(|()| self.hit(fsync))
            .and_then(|()| writer.get_ref().sync_all().map_err(store_io));
        if let Err(error) = write_result {
            return Err(cleanup_temporary(&path, writer.get_ref())
                .err()
                .unwrap_or(error));
        }
        let file = match writer.into_inner() {
            Ok(file) => file,
            Err(error) => {
                let writer = error.into_inner();
                return Err(cleanup_temporary(&path, writer.get_ref())
                    .err()
                    .unwrap_or_else(store_error));
            }
        };
        Ok(StagedFile::new(path, file))
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

#[derive(Debug)]
pub(super) enum CommitFailure {
    BeforeYaml(MemoryError),
    AfterYaml,
}

fn store_io(_: std::io::Error) -> MemoryError {
    store_error()
}

const fn store_error() -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}

fn cleanup_before_yaml(yaml: StagedFile, index: StagedIndex, error: MemoryError) -> CommitFailure {
    let index_cleanup = index.discard();
    let yaml_cleanup = yaml.discard();
    CommitFailure::BeforeYaml(
        index_cleanup
            .err()
            .or_else(|| yaml_cleanup.err())
            .unwrap_or(error),
    )
}

fn cleanup_derived(staged: StagedIndex, error: MemoryError) -> MemoryError {
    staged.discard().err().unwrap_or(error)
}
