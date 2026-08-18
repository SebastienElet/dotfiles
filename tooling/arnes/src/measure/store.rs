use super::MeasureError;
use super::model::RunRecord;
use serde::Serialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

mod path;
mod serialization;
#[cfg(test)]
mod tests;
pub(super) mod validation;

pub(super) use path::ManagedPath;
#[cfg(test)]
pub use serialization::{append_jsonl, write_json_atomic_test};
pub(super) use serialization::{
    append_jsonl_bytes, compact_json_bytes, json_bytes, jsonl_bytes, write_json_atomic_bytes,
};

pub(super) const MAX_RECORD_BYTES: usize = 1_100_000;

pub struct Store {
    root: ManagedPath,
}

impl Store {
    pub fn open(repositories: &[PathBuf]) -> Result<Self, MeasureError> {
        let resolved = resolve_candidate(&state_base()?)?.join("dotfiles/agent-harness");
        reject_repositories(&resolved, repositories)?;
        let directory = path::open_root(&resolved)?;
        Ok(Self {
            root: ManagedPath::root(directory, resolved),
        })
    }

    pub fn run_dir(&self, run_id: &str) -> Result<ManagedPath, MeasureError> {
        let run = self.run_path(run_id);
        for path in [
            self.runs_path(),
            run.clone(),
            run.join("artifacts"),
            run.join("artifacts/hooks"),
        ] {
            path.create_dir_all()?;
        }
        Ok(run)
    }

    pub fn run_path(&self, run_id: &str) -> ManagedPath {
        self.root.join("runs").join(run_id)
    }

    pub fn runs_path(&self) -> ManagedPath {
        self.root.join("runs")
    }

    pub fn append_invalid<T: Serialize>(&self, record: &T) -> Result<(), MeasureError> {
        let bytes = jsonl_bytes(record)?;
        append_jsonl_bytes(&self.root.join("invalid.jsonl"), &bytes)
    }
}

pub fn write_json_once(
    path: &ManagedPath,
    value: Option<&RunRecord>,
    agent: &str,
    session: &str,
    run_id: &str,
) -> Result<(), MeasureError> {
    let lock = open_private_append(&path.join_extension("json.lock"))?;
    lock.lock()?;
    if path.exists()? {
        let current = validation::read_run(path)?;
        validation::validate_run(&current, agent, session, run_id)
    } else {
        let value = value.ok_or_else(|| MeasureError::new("managed run.json disappeared"))?;
        serialization::write_json_atomic(path, value)
    }
}

pub(super) fn open_private_append(path: &ManagedPath) -> Result<std::fs::File, MeasureError> {
    path.open_append()
}

pub(super) fn open_private_new(path: &ManagedPath) -> Result<std::fs::File, MeasureError> {
    path.open_new()
}

pub(super) fn temporary_path(path: &ManagedPath) -> ManagedPath {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.join_extension(format!("tmp-{}-{nanos}", std::process::id()))
}

fn state_base() -> Result<PathBuf, MeasureError> {
    let base = match env::var_os("XDG_STATE_HOME") {
        Some(path) => checked_absolute(path, "XDG_STATE_HOME")?,
        None => checked_absolute(
            env::var_os("HOME").ok_or_else(|| MeasureError::new("HOME is required"))?,
            "HOME",
        )?
        .join(".local/state"),
    };
    Ok(base)
}

fn checked_absolute(value: std::ffi::OsString, name: &str) -> Result<PathBuf, MeasureError> {
    let path = PathBuf::from(value);
    if path.as_os_str().is_empty() {
        return Err(MeasureError::new(format!("{name} cannot be empty")));
    }
    if !path.is_absolute() {
        return Err(MeasureError::new(format!(
            "{name} must be an absolute path"
        )));
    }
    Ok(path)
}

fn reject_repositories(root: &Path, repositories: &[PathBuf]) -> Result<(), MeasureError> {
    for repository in repositories {
        let repository = fs::canonicalize(repository)?;
        if root.starts_with(&repository) {
            return Err(MeasureError::new(
                "state root cannot resolve inside the repository",
            ));
        }
    }
    Ok(())
}

fn resolve_candidate(path: &Path) -> Result<PathBuf, MeasureError> {
    let mut existing = path;
    let mut suffix = Vec::new();
    while !existing.exists() {
        suffix.push(
            existing
                .file_name()
                .ok_or_else(|| MeasureError::new("invalid state root"))?,
        );
        existing = existing
            .parent()
            .ok_or_else(|| MeasureError::new("invalid state root"))?;
    }
    let mut resolved = fs::canonicalize(existing)?;
    for component in suffix.into_iter().rev() {
        resolved.push(component);
    }
    Ok(resolved)
}
