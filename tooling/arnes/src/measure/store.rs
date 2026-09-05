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
pub(super) use serialization::write_json_atomic_bytes;
#[cfg(test)]
pub use serialization::{append_jsonl, write_json_atomic_test};
pub(super) use serialization::{append_jsonl_bytes, json_bytes, jsonl_bytes, write_json_atomic};

pub(super) const MAX_RECORD_BYTES: usize = 1_100_000;

pub struct Store {
    root: ManagedPath,
}

#[derive(Serialize)]
pub struct StorageUsage {
    pub logical_bytes: u64,
    pub allocated_bytes: u64,
}

impl Store {
    pub fn open(repositories: &[PathBuf]) -> Result<Self, MeasureError> {
        Self::open_from_state_base(&state_base()?, repositories)
    }

    fn open_from_state_base(
        state_base: &Path,
        repositories: &[PathBuf],
    ) -> Result<Self, MeasureError> {
        let resolved = resolve_candidate(state_base)?.join("dotfiles/agent-harness");
        reject_repositories(&resolved, repositories)?;
        let directory = path::open_root(&resolved)?;
        Ok(Self {
            root: ManagedPath::root(directory, resolved),
        })
    }

    pub fn run_dir(&self, run_id: &str) -> Result<ManagedPath, MeasureError> {
        let run = self.run_path(run_id);
        for path in [self.runs_path(), run.clone()] {
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

    pub fn open_run_lock(&self, run_id: &str) -> Result<std::fs::File, MeasureError> {
        let slot = run_id.chars().take(2).collect::<String>();
        if slot.len() != 2 || !slot.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(MeasureError::new("managed run id has no lock slot"));
        }
        let locks = self.root.join("run-locks");
        locks.create_dir_all()?;
        open_private_append(&locks.join(format!("{slot}.lock")))
    }

    pub fn append_invalid<T: Serialize>(&self, record: &T) -> Result<(), MeasureError> {
        let bytes = jsonl_bytes(record)?;
        append_jsonl_bytes(&self.root.join("invalid.jsonl"), &bytes)
    }

    pub fn usage(&self) -> Result<StorageUsage, MeasureError> {
        usage(&self.root)
    }

    pub(super) fn state_path(&self, name: &str) -> ManagedPath {
        self.root.join(name)
    }
}

fn usage(path: &ManagedPath) -> Result<StorageUsage, MeasureError> {
    use std::os::unix::fs::MetadataExt;

    let directory = path.open_directory()?;
    let mut total = StorageUsage {
        logical_bytes: 0,
        allocated_bytes: directory.metadata()?.blocks() * 512,
    };
    for name in path.read_dir_names()? {
        let child = path.join(name);
        if child.open_directory().is_ok() {
            let nested = usage(&child)?;
            total.logical_bytes = total.logical_bytes.saturating_add(nested.logical_bytes);
            total.allocated_bytes = total.allocated_bytes.saturating_add(nested.allocated_bytes);
        } else {
            let metadata = child.open_read()?.metadata()?;
            total.logical_bytes = total.logical_bytes.saturating_add(metadata.len());
            total.allocated_bytes = total
                .allocated_bytes
                .saturating_add(metadata.blocks() * 512);
        }
    }
    Ok(total)
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
