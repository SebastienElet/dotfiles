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
        return Ok(path.into());
    }
    environment
        .home
        .as_deref()
        .map(|home| Path::new(home).join(".local/state"))
        .ok_or_else(|| HandoffError::usage("missing HOME and XDG_STATE_HOME"))
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
