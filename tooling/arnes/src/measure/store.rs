use super::MeasureError;
mod validation;
use self::validation::{
    ensure_regular_file, ensure_regular_or_missing, ensure_single_link, read_run, validate_jsonl,
};
use super::model::RunRecord;
use serde::Serialize;
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn open(repositories: &[PathBuf]) -> Result<Self, MeasureError> {
        let root = state_root()?;
        let resolved = resolve_candidate(&root)?;
        for repository in repositories {
            let repository = fs::canonicalize(repository)?;
            if resolved.starts_with(&repository) {
                return Err(MeasureError::new(
                    "state root cannot resolve inside the repository",
                ));
            }
        }
        ensure_state_base(root.parent().unwrap().parent().unwrap())?;
        create_private_dir(root.parent().unwrap())?;
        create_private_dir(&root)?;
        Ok(Self { root })
    }

    pub fn run_dir(&self, run_id: &str) -> Result<PathBuf, MeasureError> {
        let run = self.run_path(run_id);
        let runs = run.parent().unwrap();
        create_private_dir(runs)?;
        create_private_dir(&run)?;
        create_private_dir(&run.join("artifacts"))?;
        create_private_dir(&run.join("artifacts/hooks"))?;
        Ok(run)
    }

    pub fn run_path(&self, run_id: &str) -> PathBuf {
        self.root.join("runs").join(run_id)
    }

    pub fn append_invalid<T: Serialize>(&self, record: &T) -> Result<(), MeasureError> {
        append_jsonl(&self.root.join("invalid.jsonl"), record)
    }
}

pub fn append_jsonl<T: Serialize>(path: &Path, value: &T) -> Result<(), MeasureError> {
    let mut bytes = serde_json::to_vec(value)?;
    bytes.push(b'\n');
    let mut file = open_private_append(path)?;
    file.lock()?;
    validate_jsonl(&mut file)?;
    file.write_all(&bytes)?;
    file.sync_data()?;
    Ok(())
}

pub fn write_json_once(
    path: &Path,
    value: Option<&RunRecord>,
    agent: &str,
    session: &str,
    run_id: &str,
) -> Result<(), MeasureError> {
    let lock_path = path.with_extension("json.lock");
    let lock = open_private_append(&lock_path)?;
    lock.lock()?;
    match fs::symlink_metadata(path) {
        Ok(_) => {
            ensure_regular_file(path)?;
            let current = read_run(path)?;
            validation::validate_run(&current, agent, session, run_id)?;
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            let value = value.ok_or_else(|| MeasureError::new("managed run.json disappeared"))?;
            write_json_atomic(path, value)
        }
        Err(error) => Err(error.into()),
    }
}

pub fn write_json_atomic<T: Serialize>(path: &Path, value: &T) -> Result<(), MeasureError> {
    let bytes = serde_json::to_vec_pretty(value)?;
    let temporary = temporary_path(path);
    let mut file = open_private_new(&temporary)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    fs::rename(&temporary, path)?;
    Ok(())
}

fn state_root() -> Result<PathBuf, MeasureError> {
    let base = match env::var_os("XDG_STATE_HOME") {
        Some(path) => checked_absolute(path, "XDG_STATE_HOME")?,
        None => checked_absolute(
            env::var_os("HOME").ok_or_else(|| MeasureError::new("HOME is required"))?,
            "HOME",
        )?
        .join(".local/state"),
    };
    Ok(base.join("dotfiles/agent-harness"))
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

fn ensure_state_base(path: &Path) -> Result<(), MeasureError> {
    if path.exists() {
        return ensure_directory(path);
    }
    let mut missing = Vec::new();
    let mut existing = path;
    while !existing.exists() {
        missing.push(existing);
        existing = existing
            .parent()
            .ok_or_else(|| MeasureError::new("state base has no existing ancestor"))?;
    }
    ensure_directory(existing)?;
    for directory in missing.into_iter().rev() {
        create_private_dir(directory)?;
    }
    Ok(())
}

fn create_private_dir(path: &Path) -> Result<(), MeasureError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if !metadata.is_dir() || metadata.file_type().is_symlink() => {
            return Err(MeasureError::new(format!(
                "managed path is not a real directory: {}",
                path.display()
            )));
        }
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => match fs::create_dir(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                let metadata = fs::symlink_metadata(path)?;
                if !metadata.is_dir() || metadata.file_type().is_symlink() {
                    return Err(MeasureError::new(format!(
                        "managed path is not a real directory: {}",
                        path.display()
                    )));
                }
            }
            Err(error) => return Err(error.into()),
        },
        Err(error) => return Err(error.into()),
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn ensure_directory(path: &Path) -> Result<(), MeasureError> {
    let metadata = fs::metadata(path)?;
    if !metadata.is_dir() {
        return Err(MeasureError::new(format!(
            "state path is not a directory: {}",
            path.display()
        )));
    }
    Ok(())
}

fn open_private_append(path: &Path) -> Result<File, MeasureError> {
    ensure_regular_or_missing(path)?;
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .append(true)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    ensure_single_link(&file.metadata()?, path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
}

fn open_private_new(path: &Path) -> Result<File, MeasureError> {
    let file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(rustix::fs::OFlags::NOFOLLOW.bits() as i32)
        .open(path)?;
    file.set_permissions(fs::Permissions::from_mode(0o600))?;
    Ok(file)
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

fn temporary_path(path: &Path) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    path.with_extension(format!("tmp-{}-{nanos}", std::process::id()))
}
