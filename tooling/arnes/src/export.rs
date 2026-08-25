use crate::Roots;
use sha2::{Digest, Sha256};
use std::fmt::{self, Display};
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

mod files;
mod render;
mod sources;

use files::{check_snapshot, publish_snapshot, validate_export_directory};
use render::render_snapshot;
use sources::read_sources;

const EXPORT_DIRECTORY: &str = ".harness-export";

#[derive(Clone, Debug, Eq, PartialEq)]
struct Metadata {
    commit: String,
    generated_at: u64,
    repository_state: String,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ExportError(String);

impl ExportError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl Display for ExportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ExportError {}

pub fn run(roots: &Roots, check: bool) -> Result<(), ExportError> {
    let output = roots.repository().join(EXPORT_DIRECTORY);
    let metadata = if check {
        validate_export_directory(&output)?;
        render::read_metadata(&output.join("00-MANIFEST.md"))?
    } else {
        current_metadata(roots.repository())?
    };
    let sources = read_sources(roots.repository())?;
    let snapshot = render_snapshot(&sources, &metadata);
    if check {
        check_snapshot(&output, &snapshot)
    } else {
        publish_snapshot(&output, &snapshot)
    }
}

fn current_metadata(repository: &Path) -> Result<Metadata, ExportError> {
    let commit = current_commit(repository)?;
    let status = run_git(
        repository,
        &[
            "status",
            "--porcelain=v1",
            "--untracked-files=normal",
            "--",
            ".",
            ":(exclude).harness-export",
        ],
    )?;
    let generated_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| ExportError::new("system clock is before the Unix epoch"))?
        .as_secs();
    Ok(Metadata {
        commit,
        generated_at,
        repository_state: if status.is_empty() { "clean" } else { "dirty" }.to_owned(),
    })
}

fn current_commit(repository: &Path) -> Result<String, ExportError> {
    let output = Command::new("git")
        .args(["rev-parse", "--verify", "HEAD"])
        .current_dir(repository)
        .output()
        .map_err(|error| ExportError::new(format!("Git could not run: {error}")))?;
    if !output.status.success() {
        return Ok("unavailable".to_owned());
    }
    let commit = String::from_utf8(output.stdout)
        .map_err(|_| ExportError::new("Git returned a non-UTF-8 commit"))?
        .trim()
        .to_owned();
    if commit.is_empty() || !commit.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ExportError::new("Git returned an invalid commit"));
    }
    Ok(commit)
}

pub(super) fn run_git(repository: &Path, arguments: &[&str]) -> Result<Vec<u8>, ExportError> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(repository)
        .output()
        .map_err(|error| ExportError::new(format!("Git could not run: {error}")))?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(ExportError::new(if detail.is_empty() {
            format!("Git failed with status {}", output.status)
        } else {
            format!("Git failed: {detail}")
        }));
    }
    Ok(output.stdout)
}

fn sha256(contents: &str) -> String {
    format!("{:x}", Sha256::digest(contents.as_bytes()))
}
