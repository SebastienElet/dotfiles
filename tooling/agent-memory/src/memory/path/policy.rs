use super::component::{private_directory_mode, repair_mode, validate_mode};
use crate::memory::MemoryError;
use std::fs::File;

#[derive(Clone, Copy, Debug)]
pub(crate) enum DirectoryAccess {
    Repair,
    Validate,
}

impl DirectoryAccess {
    pub(super) fn enforce(self, directory: &File) -> Result<(), MemoryError> {
        match self {
            Self::Repair => repair_mode(directory, private_directory_mode(), false),
            Self::Validate => validate_mode(directory, private_directory_mode()),
        }
    }
}
