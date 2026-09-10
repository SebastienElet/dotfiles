use crate::{Environment, HandoffError};
use std::ffi::{OsStr, OsString};
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::os::unix::ffi::{OsStrExt, OsStringExt};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SentinelState {
    Created,
    Existing,
}

/// # Errors
/// Returns a usage error when neither a nonempty XDG state root nor HOME is available.
pub fn state_root(environment: &Environment) -> Result<PathBuf, HandoffError> {
    if let Some(path) = environment
        .xdg_state_home
        .as_deref()
        .filter(|path| !path.is_empty())
    {
        return Ok(join_posix(&[path]));
    }
    environment
        .home
        .as_deref()
        .map(|home| join_posix(&[home, OsStr::new(".local"), OsStr::new("state")]))
        .ok_or_else(|| HandoffError::usage("missing HOME and XDG_STATE_HOME"))
}

pub fn join_posix(paths: &[impl AsRef<OsStr>]) -> PathBuf {
    let mut joined = Vec::new();
    for path in paths
        .iter()
        .map(|path| path.as_ref().as_bytes())
        .filter(|path| !path.is_empty())
    {
        if !joined.is_empty() {
            joined.push(b'/');
        }
        joined.extend_from_slice(path);
    }
    if joined.is_empty() {
        return PathBuf::from(".");
    }

    let absolute = joined.starts_with(b"/");
    let trailing_separator = joined.ends_with(b"/");
    let mut components = Vec::new();
    for component in joined.split(|byte| *byte == b'/') {
        match component {
            b".." if components.last().is_some_and(|last| *last != b"..") => {
                components.pop();
            }
            b".." if !absolute => components.push(component),
            b"" | b"." | b".." => {}
            _ => components.push(component),
        }
    }

    let mut normalized = components.join(&b'/');
    if absolute {
        normalized.insert(0, b'/');
    } else if normalized.is_empty() {
        normalized.push(b'.');
    }
    if trailing_separator && normalized != b"/" {
        normalized.push(b'/');
    }
    PathBuf::from(OsString::from_vec(normalized))
}

/// # Errors
/// Returns an unexpected error if the sentinel is not a regular file or cannot be inspected.
pub fn inspect_sentinel(path: &Path) -> Result<bool, HandoffError> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Ok(_) | Err(_) => Err(HandoffError::unexpected("cannot inspect handoff sentinel")),
    }
}

/// # Errors
/// Returns an unexpected error if the parent directory or sentinel cannot be created.
pub fn create_sentinel(path: &Path) -> Result<SentinelState, HandoffError> {
    let parent = path
        .parent()
        .ok_or_else(|| HandoffError::unexpected("cannot create handoff sentinel"))?;
    fs::create_dir_all(parent)
        .map_err(|_| HandoffError::unexpected("cannot create handoff sentinel"))?;
    match OpenOptions::new().write(true).create_new(true).open(path) {
        Ok(_) => Ok(SentinelState::Created),
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(SentinelState::Existing),
        Err(_) => Err(HandoffError::unexpected("cannot create handoff sentinel")),
    }
}

#[cfg(test)]
mod tests {
    use super::join_posix;
    use std::path::PathBuf;

    #[test]
    fn posix_join_matches_node_lexical_normalization() {
        let cases: &[(&[&str], &str)] = &[
            (&[""], "."),
            (&["/"], "/"),
            (&["//"], "/"),
            (&["foo/"], "foo/"),
            (&["foo", ""], "foo"),
            (&["foo", "."], "foo"),
            (&["foo", ".."], "."),
            (&["foo/../"], "./"),
            (&["foo/../..", "bar"], "../bar"),
            (&["/foo/../..", "bar"], "/bar"),
            (&["a//b/./c/../", "d"], "a/b/d"),
            (&["../a", "../b"], "../b"),
            (&["a/../../"], "../"),
        ];

        for (paths, expected) in cases {
            assert_eq!(join_posix(paths), PathBuf::from(expected), "{paths:?}");
        }
    }
}
