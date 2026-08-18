use super::model::RepositoryRecord;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn observe(directory: &Path) -> Option<RepositoryRecord> {
    let root = git(directory, &["rev-parse", "--show-toplevel"])?;
    let head = git(directory, &["rev-parse", "HEAD"]);
    let branch = git(directory, &["branch", "--show-current"]).filter(|value| !value.is_empty());
    let dirty = git(directory, &["status", "--porcelain"])
        .map(|value| !value.is_empty())
        .unwrap_or(true);
    Some(RepositoryRecord {
        root,
        head,
        branch,
        dirty,
    })
}

pub fn protected_root(directory: &Path) -> PathBuf {
    directory
        .ancestors()
        .filter(|ancestor| fs::symlink_metadata(ancestor.join(".git")).is_ok())
        .last()
        .unwrap_or(directory)
        .to_owned()
}

fn git(directory: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(directory)
        .output()
        .ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
}
