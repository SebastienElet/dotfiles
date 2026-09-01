use crate::{Environment, HandoffError};
use std::fs::{self, OpenOptions};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SentinelState {
    Created,
    Existing,
}

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
        .map(|home| join_posix(&[home, ".local", "state"]))
        .ok_or_else(|| HandoffError::usage("missing HOME and XDG_STATE_HOME"))
}

pub(crate) fn join_posix(paths: &[&str]) -> PathBuf {
    let mut joined = String::new();
    for path in paths.iter().filter(|path| !path.is_empty()) {
        if !joined.is_empty() {
            joined.push('/');
        }
        joined.push_str(path);
    }
    if joined.is_empty() {
        return PathBuf::from(".");
    }

    let absolute = joined.starts_with('/');
    let trailing_separator = joined.ends_with('/');
    let mut components = Vec::new();
    for component in joined.split('/') {
        match component {
            "" | "." => {}
            ".." if components.last().is_some_and(|last| *last != "..") => {
                components.pop();
            }
            ".." if !absolute => components.push(component),
            ".." => {}
            _ => components.push(component),
        }
    }

    let mut normalized = components.join("/");
    if absolute {
        normalized.insert(0, '/');
    } else if normalized.is_empty() {
        normalized.push('.');
    }
    if trailing_separator && normalized != "/" {
        normalized.push('/');
    }
    PathBuf::from(normalized)
}

pub fn inspect_sentinel(path: &Path) -> Result<bool, HandoffError> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_file() => Ok(true),
        Ok(_) => Err(HandoffError::unexpected("cannot inspect handoff sentinel")),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(_) => Err(HandoffError::unexpected("cannot inspect handoff sentinel")),
    }
}

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
