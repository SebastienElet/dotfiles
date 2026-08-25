use super::{ExportError, run_git};
mod security;
use security::{reject_ignored_harness_paths, reject_sensitive_path};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) enum Category {
    Instructions,
    CommandsRules,
    Skills,
    HooksRouting,
    Services,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Source {
    pub(super) category: Category,
    pub(super) contents: String,
    pub(super) kind: String,
    pub(super) path: String,
}

pub(super) fn read_sources(repository: &Path) -> Result<Vec<Source>, ExportError> {
    reject_ignored_harness_paths(repository)?;
    let classified = classify_paths(git_paths(repository)?)?;
    let selected = classified.keys().cloned().collect::<BTreeSet<_>>();
    classified
        .into_iter()
        .map(|(path, category)| read_source(repository, &selected, path, category))
        .collect()
}

fn read_source(
    repository: &Path,
    selected: &BTreeSet<String>,
    path: String,
    category: Category,
) -> Result<Source, ExportError> {
    let absolute = repository.join(&path);
    let metadata = fs::symlink_metadata(&absolute).map_err(|error| {
        ExportError::new(format!(
            "source {path} is missing or unreadable; export is stale: {error}"
        ))
    })?;
    let (read_path, kind) = if metadata.file_type().is_symlink() {
        read_symlink_source(repository, selected, &absolute, &path)?
    } else if metadata.file_type().is_file() && metadata.nlink() == 1 {
        (absolute, "file".to_owned())
    } else if metadata.file_type().is_file() {
        return Err(ExportError::new(format!(
            "refusing hardlinked source {path}"
        )));
    } else {
        return Err(ExportError::new(format!(
            "refusing non-regular source {path}"
        )));
    };
    let bytes = fs::read(&read_path)
        .map_err(|error| ExportError::new(format!("source {path} could not be read: {error}")))?;
    let contents = String::from_utf8(bytes)
        .map_err(|_| ExportError::new(format!("source {path} is not valid UTF-8")))?;
    Ok(Source {
        category,
        contents,
        kind,
        path,
    })
}

fn read_symlink_source(
    repository: &Path,
    selected: &BTreeSet<String>,
    absolute: &Path,
    path: &str,
) -> Result<(PathBuf, String), ExportError> {
    let target = fs::read_link(absolute)
        .map_err(|error| ExportError::new(format!("source {path} link is unreadable: {error}")))?;
    let resolved = fs::canonicalize(absolute).map_err(|error| {
        ExportError::new(format!("source {path} link target is unreadable: {error}"))
    })?;
    let harness = fs::canonicalize(repository.join("harness"))
        .map_err(|error| ExportError::new(format!("harness source root is unreadable: {error}")))?;
    if !resolved.starts_with(&harness) {
        return Err(ExportError::new(format!(
            "refusing source link outside harness: {path}"
        )));
    }
    let repository = fs::canonicalize(repository)
        .map_err(|error| ExportError::new(format!("repository is unreadable: {error}")))?;
    let relative = resolved
        .strip_prefix(&repository)
        .ok()
        .and_then(Path::to_str)
        .ok_or_else(|| ExportError::new(format!("source {path} link target is invalid")))?;
    if !selected.contains(relative) {
        return Err(ExportError::new(format!(
            "refusing source link to an unselected target: {path}"
        )));
    }
    let metadata = fs::metadata(&resolved).map_err(|error| {
        ExportError::new(format!("source {path} link target is unreadable: {error}"))
    })?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err(ExportError::new(format!(
            "refusing non-owned source link target: {path}"
        )));
    }
    Ok((resolved, format!("symlink -> {}", target.display())))
}

fn git_paths(repository: &Path) -> Result<Vec<String>, ExportError> {
    let output = run_git(
        repository,
        &[
            "ls-files",
            "--cached",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            "harness",
            "home/.arnes.yaml",
        ],
    )?;
    let paths = decode_git_paths(output)?;
    if paths.is_empty() {
        return Err(ExportError::new("Git returned no harness source paths"));
    }
    Ok(paths)
}

fn decode_git_paths(output: Vec<u8>) -> Result<Vec<String>, ExportError> {
    if output.is_empty() {
        return Ok(Vec::new());
    }
    if !output.ends_with(&[0]) {
        return Err(ExportError::new("Git returned a malformed path list"));
    }
    let decoded = String::from_utf8(output)
        .map_err(|_| ExportError::new("Git returned non-UTF-8 repository paths"))?;
    decoded[..decoded.len() - 1]
        .split('\0')
        .map(|path| {
            if path.is_empty()
                || path.starts_with('/')
                || path.split('/').any(|part| part.is_empty() || part == "..")
                || path.contains(['|', '`', '\n', '\r'])
            {
                Err(ExportError::new(format!(
                    "Git returned unsafe repository path {path:?}"
                )))
            } else {
                Ok(path.to_owned())
            }
        })
        .collect()
}

fn classify_paths(paths: Vec<String>) -> Result<BTreeMap<String, Category>, ExportError> {
    let required = BTreeSet::from([
        "harness/AGENTS.md",
        "harness/SOUL.md",
        "harness/USER.md",
        "home/.arnes.yaml",
    ]);
    let mut sources = BTreeMap::new();
    for path in paths {
        if path.ends_with("/.DS_Store") || path == "harness/skills/README.md" {
            continue;
        }
        reject_sensitive_path(&path)?;
        let category = if matches!(
            path.as_str(),
            "harness/AGENTS.md" | "harness/SOUL.md" | "harness/USER.md"
        ) {
            Category::Instructions
        } else if path.starts_with("harness/commands/") || path.starts_with("harness/rules/") {
            Category::CommandsRules
        } else if path.starts_with("harness/skills/") {
            Category::Skills
        } else if path == "home/.arnes.yaml" {
            Category::HooksRouting
        } else if path.starts_with("harness/") {
            Category::Services
        } else {
            continue;
        };
        sources.insert(path, category);
    }
    for path in required {
        if !sources.contains_key(path) {
            return Err(ExportError::new(format!(
                "required harness source {path} is missing"
            )));
        }
    }
    Ok(sources)
}
