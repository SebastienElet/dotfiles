use super::MemoryError;
use std::env;
use std::path::{Component, Path, PathBuf};

mod access;
mod component;
mod policy;

pub use access::ManagedPath;
pub use component::{open_existing_root, open_root};
pub use policy::DirectoryAccess;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MemoryRoot(PathBuf);

impl MemoryRoot {
    /// # Errors
    ///
    /// Returns an error if neither root nor home is available, or the selected root path is invalid or inaccessible.
    pub fn from_environment() -> Result<Self, MemoryError> {
        if let Some(path) = env::var_os("AGENT_MEMORY_ROOT") {
            Self::new(PathBuf::from(path))
        } else {
            let home = env::var_os("HOME")
                .ok_or_else(|| MemoryError::unavailable("memory_root_unavailable", "store"))?;
            Self::new(PathBuf::from(home).join(".local/share/agent-memory"))
        }
    }

    /// # Errors
    ///
    /// Returns an error for a nonabsolute path, parent traversal, a missing final name, or an inaccessible existing ancestor.
    pub fn new(path: impl AsRef<Path>) -> Result<Self, MemoryError> {
        let path = path.as_ref();
        if !path.is_absolute()
            || path
                .components()
                .any(|component| !matches!(component, Component::RootDir | Component::Normal(_)))
        {
            return Err(unsafe_path());
        }
        let name = path.file_name().ok_or_else(unsafe_path)?;
        let parent = path.parent().ok_or_else(unsafe_path)?;
        let resolved_parent = resolve_missing_parent(parent)?;
        Ok(Self(resolved_parent.join(name)))
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.0
    }
}

fn resolve_missing_parent(path: &Path) -> Result<PathBuf, MemoryError> {
    let mut existing = path;
    let mut missing = Vec::new();
    while !existing.exists() {
        missing.push(existing.file_name().ok_or_else(unsafe_path)?);
        existing = existing.parent().ok_or_else(unsafe_path)?;
    }
    let mut resolved = existing.canonicalize().map_err(store_unavailable)?;
    for component in missing.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}

const fn unsafe_path() -> MemoryError {
    MemoryError::unavailable("unsafe_store_path", "store")
}

fn store_unavailable(_: std::io::Error) -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}
