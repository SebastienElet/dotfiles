use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{Result, ensure, model::State};

pub const MAX_BYTES: u64 = 1_048_576;

pub fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub fn private_directory(path: &Path) -> Result<PathBuf> {
    let metadata = fs::symlink_metadata(path)?;
    ensure(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "private directory must be a real directory",
    )?;
    ensure(
        metadata.permissions().mode() & 0o077 == 0,
        "private directory must have mode 0700",
    )?;
    Ok(path.canonicalize()?)
}

pub fn create_directory(path: &Path) -> Result<PathBuf> {
    fs::DirBuilder::new().mode(0o700).create(path)?;
    private_directory(path)
}

pub fn create_file(path: &Path) -> Result<File> {
    Ok(OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?)
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let metadata = fs::symlink_metadata(path)?;
    ensure(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "expected a regular private file",
    )?;
    ensure(metadata.len() <= MAX_BYTES, "input exceeds size limit")?;
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes).map_err(|_| "invalid structured input".into())
}

pub fn read_input<T: DeserializeOwned>() -> Result<T> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure(bytes.len() as u64 <= MAX_BYTES, "input exceeds size limit")?;
    serde_json::from_slice(&bytes).map_err(|_| "invalid structured input".into())
}

pub fn write_json(path: &Path, value: &impl Serialize) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)?;
    ensure(
        bytes.len() as u64 <= MAX_BYTES,
        "state exceeds size limit; previous state preserved",
    )?;
    let mut file = create_file(path)?;
    file.write_all(&bytes)?;
    file.sync_all()?;
    Ok(())
}

pub fn save(directory: &Path, state: &State) -> Result<()> {
    let temporary = directory.join("state.next");
    if temporary.exists() {
        ensure(
            fs::symlink_metadata(&temporary)?.is_file(),
            "unexpected state transaction file",
        )?;
        fs::remove_file(&temporary)?;
    }
    write_json(&temporary, state)?;
    fs::rename(temporary, directory.join("state.json"))?;
    File::open(directory)?.sync_all()?;
    Ok(())
}

pub fn lock(path: &Path) -> Result<File> {
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .open(path)?;
    file.try_lock()
        .map_err(|_| "another supervisor owns this task or worktree")?;
    Ok(file)
}

pub fn git(repository: &Path, arguments: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(repository)
        .args(arguments)
        .output()?;
    ensure(output.status.success(), "candidate repository unavailable")?;
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
