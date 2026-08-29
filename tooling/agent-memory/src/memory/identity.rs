use super::{MemoryError, ProcessRunner, ProjectKey};
use sha2::{Digest, Sha256};
use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Eq, PartialEq)]
pub struct ProjectScope {
    key: ProjectKey,
    common_directory: PathBuf,
}

impl ProjectScope {
    pub fn key(&self) -> &ProjectKey {
        &self.key
    }

    pub fn common_directory(&self) -> &Path {
        &self.common_directory
    }
}

pub fn resolve_project(cwd: &Path, git: &dyn ProcessRunner) -> Result<ProjectScope, MemoryError> {
    let common_directory = resolve_git_directory(cwd, git, "--git-common-dir")?;
    let hash = Sha256::digest(common_directory.as_os_str().as_bytes());
    Ok(ProjectScope {
        key: ProjectKey::from_validated(format!("project_{hash:x}")),
        common_directory,
    })
}

pub(crate) fn resolve_worktree_directory(
    cwd: &Path,
    git: &dyn ProcessRunner,
) -> Result<PathBuf, MemoryError> {
    resolve_git_directory(cwd, git, "--show-toplevel")
}

fn resolve_git_directory(
    cwd: &Path,
    git: &dyn ProcessRunner,
    selector: &str,
) -> Result<PathBuf, MemoryError> {
    let arguments = [
        OsString::from("rev-parse"),
        OsString::from("--path-format=absolute"),
        OsString::from(selector),
    ];
    let output = git
        .run(OsStr::new("git"), &arguments, Some(cwd))
        .map_err(|_| scope_unavailable())?;
    if !output.success() {
        return Err(scope_unavailable());
    }
    let common_directory = single_absolute_path(output.stdout())?
        .canonicalize()
        .map_err(|_| scope_unavailable())?;
    if !common_directory.is_absolute() || !common_directory.is_dir() {
        return Err(scope_unavailable());
    }
    Ok(common_directory)
}

fn single_absolute_path(bytes: &[u8]) -> Result<PathBuf, MemoryError> {
    let value = std::str::from_utf8(bytes).map_err(|_| scope_unavailable())?;
    let value = value.strip_suffix('\n').unwrap_or(value);
    let value = value.strip_suffix('\r').unwrap_or(value);
    if value.is_empty() || value.contains(['\n', '\r']) {
        return Err(scope_unavailable());
    }
    let path = PathBuf::from(value);
    if !path.is_absolute() {
        return Err(scope_unavailable());
    }
    Ok(path)
}

const fn scope_unavailable() -> MemoryError {
    MemoryError::unavailable("scope_unavailable", "scope")
}
