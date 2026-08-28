use super::StoreFailpoint;
use crate::memory::MemoryError;
use crate::memory::path::ManagedPath;
use std::fs::Metadata;
use std::io::{BufWriter, Write};

pub(super) struct AtomicPublication<'a> {
    root: &'a ManagedPath,
    failpoint: Option<StoreFailpoint>,
}

impl<'a> AtomicPublication<'a> {
    pub(super) fn new(root: &'a ManagedPath, failpoint: Option<StoreFailpoint>) -> Self {
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
    ) -> Result<StagedYaml, MemoryError> {
        let (path, metadata) = self.write_temporary(
            destination,
            bytes,
            StoreFailpoint::BeforeYamlTemporaryCreate,
            StoreFailpoint::BeforeYamlWrite,
            StoreFailpoint::BeforeYamlFlush,
            StoreFailpoint::BeforeYamlFsync,
        )?;
        Ok(StagedYaml { path, metadata })
    }

    pub(super) fn stage_index(&self, bytes: &[u8]) -> Result<ManagedPath, MemoryError> {
        let destination = self.root.join("index.json")?;
        destination.open_read()?;
        self.write_temporary(
            &destination,
            bytes,
            StoreFailpoint::BeforeIndexTemporaryCreate,
            StoreFailpoint::BeforeIndexWrite,
            StoreFailpoint::BeforeIndexFlush,
            StoreFailpoint::BeforeIndexFsync,
        )
        .map(|(path, _)| path)
    }

    pub(super) fn publish(
        &self,
        yaml: StagedYaml,
        destination: &ManagedPath,
        index: ManagedPath,
        replace: bool,
    ) -> Result<(), CommitFailure> {
        self.hit(StoreFailpoint::BeforeYamlRename)
            .map_err(CommitFailure::BeforeYaml)?;
        if replace {
            yaml.path.rename_to(destination)
        } else {
            yaml.path.rename_new_to(destination)
        }
        .map_err(CommitFailure::BeforeYaml)?;
        self.hit(StoreFailpoint::AfterYamlRename)
            .map_err(|_| CommitFailure::AfterYaml)?;
        self.hit(StoreFailpoint::BeforeYamlDirectoryFsync)
            .map_err(|_| CommitFailure::AfterYaml)?;
        destination
            .sync_parent_directory()
            .map_err(|_| CommitFailure::AfterYaml)?;
        self.hit(StoreFailpoint::BeforeIndexRename)
            .map_err(|_| CommitFailure::AfterYaml)?;
        index
            .rename_to(
                &self
                    .root
                    .join("index.json")
                    .map_err(|_| CommitFailure::AfterYaml)?,
            )
            .map_err(|_| CommitFailure::AfterYaml)?;
        self.root
            .sync_directory()
            .map_err(|_| CommitFailure::AfterYaml)?;
        Ok(())
    }

    fn write_temporary(
        &self,
        destination: &ManagedPath,
        bytes: &[u8],
        create: StoreFailpoint,
        write: StoreFailpoint,
        flush: StoreFailpoint,
        fsync: StoreFailpoint,
    ) -> Result<(ManagedPath, Metadata), MemoryError> {
        self.hit(create)?;
        let temporary = self.temporary_path(destination)?;
        let file = temporary.open_new()?;
        let mut writer = BufWriter::new(file);
        self.hit(write)?;
        writer.write_all(bytes).map_err(store_io)?;
        self.hit(flush)?;
        writer.flush().map_err(store_io)?;
        self.hit(fsync)?;
        writer.get_ref().sync_all().map_err(store_io)?;
        let metadata = writer.get_ref().metadata().map_err(store_io)?;
        Ok((temporary, metadata))
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

    fn hit(&self, failpoint: StoreFailpoint) -> Result<(), MemoryError> {
        if self.failpoint == Some(failpoint) {
            Err(store_error())
        } else {
            Ok(())
        }
    }
}

pub(super) struct StagedYaml {
    path: ManagedPath,
    metadata: Metadata,
}

impl StagedYaml {
    pub(super) fn metadata(&self) -> &Metadata {
        &self.metadata
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
