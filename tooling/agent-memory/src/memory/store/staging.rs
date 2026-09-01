use crate::memory::MemoryError;
use crate::memory::path::ManagedPath;
use std::fs::File;

pub(super) struct StagedFile {
    path: ManagedPath,
    file: File,
}

pub(crate) struct StagedIndex {
    staged: StagedFile,
    destination: ManagedPath,
    original: Option<File>,
}

impl StagedFile {
    pub(super) fn new(path: ManagedPath, file: File) -> Self {
        Self { path, file }
    }

    pub(super) fn anchor(&self) -> Result<File, MemoryError> {
        self.file.try_clone().map_err(store_io)
    }

    pub(super) fn ensure_anchored(&self) -> Result<(), MemoryError> {
        self.path.ensure_same_file(&self.file)
    }

    pub(super) fn rename_to(
        &self,
        destination: &ManagedPath,
        replace: bool,
    ) -> Result<(), MemoryError> {
        if replace {
            self.path.rename_to(destination)
        } else {
            self.path.rename_new_to(destination)
        }
    }

    pub(super) fn discard(self) -> Result<(), MemoryError> {
        cleanup_temporary(&self.path, &self.file)
    }
}

impl StagedIndex {
    pub(super) fn new(
        staged: StagedFile,
        destination: ManagedPath,
        original: Option<File>,
    ) -> Self {
        Self {
            staged,
            destination,
            original,
        }
    }

    pub(super) fn ensure_anchored(&self) -> Result<(), MemoryError> {
        self.staged.ensure_anchored()?;
        match &self.original {
            Some(original) => self.destination.ensure_same_file(original),
            None if self.destination.exists()? => Err(unsafe_path()),
            None => Ok(()),
        }
    }

    pub(super) fn publish(self) -> Result<(), MemoryError> {
        match self.staged.path.rename_to(&self.destination) {
            Ok(()) => Ok(()),
            Err(error) => Err(self.staged.discard().err().unwrap_or(error)),
        }
    }

    pub(super) fn discard(self) -> Result<(), MemoryError> {
        self.staged.discard()
    }
}

pub(super) fn cleanup_temporary(path: &ManagedPath, file: &File) -> Result<(), MemoryError> {
    path.ensure_same_file(file)?;
    path.remove_file()?;
    path.sync_parent_directory()
}

fn store_io(_: std::io::Error) -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::unavailable("unsafe_store_path", "store")
}
