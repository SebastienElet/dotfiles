use super::{ExportError, run_git};
mod security;
use rustix::fs::{AtFlags, FileType, Mode, OFlags};
use security::{reject_ignored_harness_paths, reject_sensitive_path};
use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::{self, Read};
use std::os::unix::ffi::OsStrExt;
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
    let root = fs::canonicalize(repository)
        .map_err(|error| ExportError::new(format!("repository is unreadable: {error}")))?;
    let root_file = open_root(&root)?;
    classified
        .into_iter()
        .map(|(path, category)| {
            read_source(repository, &root, &root_file, &selected, path, category)
        })
        .collect()
}

fn read_source(
    repository: &Path,
    root: &Path,
    root_file: &File,
    selected: &BTreeSet<String>,
    path: String,
    category: Category,
) -> Result<Source, ExportError> {
    let (parent, resolved_parent) = validated_source_parent(repository, root, root_file, &path)?;
    read_source_from_parent(
        root,
        root_file,
        selected,
        &parent,
        &resolved_parent,
        path,
        category,
    )
}

fn read_source_from_parent(
    root: &Path,
    root_file: &File,
    selected: &BTreeSet<String>,
    parent: &File,
    resolved_parent: &Path,
    path: String,
    category: Category,
) -> Result<Source, ExportError> {
    let name = Path::new(&path)
        .file_name()
        .ok_or_else(|| ExportError::new(format!("source {path} has no name")))?;
    let metadata =
        rustix::fs::statat(parent, name, AtFlags::SYMLINK_NOFOLLOW).map_err(|error| {
            ExportError::new(format!(
                "source {path} is missing or unreadable; export is stale: {error}"
            ))
        })?;
    let (mut file, kind) = match FileType::from_raw_mode(metadata.st_mode) {
        FileType::Symlink => read_symlink_source(
            root,
            root_file,
            selected,
            parent,
            resolved_parent,
            name,
            &path,
        )?,
        FileType::RegularFile => (open_regular_file(parent, name, &path)?, "file".to_owned()),
        FileType::Directory
        | FileType::Fifo
        | FileType::Socket
        | FileType::CharacterDevice
        | FileType::BlockDevice
        | FileType::Unknown => {
            return Err(ExportError::new(format!(
                "refusing non-regular source {path}"
            )));
        }
    };
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
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

fn validated_source_parent(
    repository: &Path,
    root: &Path,
    root_file: &File,
    path: &str,
) -> Result<(File, PathBuf), ExportError> {
    let mut resolved_parent = None;
    for parent in Path::new(path).ancestors().skip(1) {
        let resolved = fs::canonicalize(repository.join(parent)).map_err(|error| {
            ExportError::new(format!("source {path} parent is unreadable: {error}"))
        })?;
        if !resolved.starts_with(root) {
            return Err(ExportError::new(format!(
                "refusing source parent outside repository: {path}"
            )));
        }
        if resolved_parent.is_none() {
            resolved_parent = Some(resolved);
        }
    }
    let resolved_parent =
        resolved_parent.ok_or_else(|| ExportError::new(format!("source {path} has no parent")))?;
    let relative = resolved_parent
        .strip_prefix(root)
        .map_err(|_| ExportError::new(format!("source {path} parent is invalid")))?;
    let parent = open_directory(root_file, relative)
        .map_err(|error| ExportError::new(format!("source {path} parent changed: {error}")))?;
    Ok((parent, resolved_parent))
}

fn read_symlink_source(
    root: &Path,
    root_file: &File,
    selected: &BTreeSet<String>,
    parent: &File,
    resolved_parent: &Path,
    name: &OsStr,
    path: &str,
) -> Result<(File, String), ExportError> {
    let target = rustix::fs::readlinkat(parent, name, Vec::new())
        .map_err(|error| ExportError::new(format!("source {path} link is unreadable: {error}")))?;
    let target = PathBuf::from(OsStr::from_bytes(target.as_bytes()));
    let resolved = fs::canonicalize(resolved_parent.join(&target)).map_err(|error| {
        ExportError::new(format!("source {path} link target is unreadable: {error}"))
    })?;
    let harness = fs::canonicalize(root.join("harness"))
        .map_err(|error| ExportError::new(format!("harness source root is unreadable: {error}")))?;
    if !resolved.starts_with(&harness) {
        return Err(ExportError::new(format!(
            "refusing source link outside harness: {path}"
        )));
    }
    let relative = resolved
        .strip_prefix(root)
        .map_err(|_| ExportError::new(format!("source {path} link target is invalid")))?;
    if !relative
        .to_str()
        .is_some_and(|relative| selected.contains(relative))
    {
        return Err(ExportError::new(format!(
            "refusing source link to an unselected target: {path}"
        )));
    }
    let target_parent = open_directory(
        root_file,
        relative
            .parent()
            .ok_or_else(|| ExportError::new(format!("source {path} link target is invalid")))?,
    )
    .map_err(|error| ExportError::new(format!("source {path} link target changed: {error}")))?;
    let target_name = relative
        .file_name()
        .ok_or_else(|| ExportError::new(format!("source {path} link target is invalid")))?;
    let file = open_regular_file(&target_parent, target_name, path)?;
    Ok((file, format!("symlink -> {}", target.display())))
}

fn open_root(root: &Path) -> Result<File, ExportError> {
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    rustix::fs::open(root, flags, Mode::empty())
        .map(File::from)
        .map_err(|error| ExportError::new(format!("repository changed: {error}")))
}

fn open_directory(root: &File, relative: &Path) -> io::Result<File> {
    let mut directory = root.try_clone()?;
    let flags = OFlags::RDONLY | OFlags::DIRECTORY | OFlags::NOFOLLOW | OFlags::CLOEXEC;
    for component in relative.components() {
        let std::path::Component::Normal(name) = component else {
            return Err(io::ErrorKind::InvalidInput.into());
        };
        directory = File::from(rustix::fs::openat(&directory, name, flags, Mode::empty())?);
    }
    Ok(directory)
}

fn open_regular_file(parent: &File, name: &OsStr, path: &str) -> Result<File, ExportError> {
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let file = rustix::fs::openat(parent, name, flags, Mode::empty())
        .map(File::from)
        .map_err(|error| ExportError::new(format!("source {path} changed: {error}")))?;
    let metadata = file
        .metadata()
        .map_err(|error| ExportError::new(format!("source {path} is unreadable: {error}")))?;
    if metadata.is_file() && metadata.nlink() == 1 {
        Ok(file)
    } else if metadata.is_file() {
        Err(ExportError::new(format!(
            "refusing hardlinked source {path}"
        )))
    } else {
        Err(ExportError::new(format!(
            "refusing non-regular source {path}"
        )))
    }
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

#[cfg(test)]
mod tests {
    use super::{
        Category, open_directory, open_regular_file, open_root, read_source_from_parent,
        validated_source_parent,
    };
    use std::collections::BTreeSet;
    use std::ffi::OsStr;
    use std::fs;
    use std::os::unix::fs::symlink;
    use std::path::Path;

    #[test]
    fn source_read_remains_inside_validated_parent_after_replacement()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let fixture = tempfile::tempdir()?;
        let repository = fixture.path().join("repository");
        let outside = fixture.path().join("outside");
        fs::create_dir_all(repository.join("harness"))?;
        fs::create_dir(&outside)?;
        fs::write(repository.join("harness/AGENTS.md"), "inside\n")?;
        fs::write(outside.join("AGENTS.md"), "outside\n")?;

        let root = fs::canonicalize(&repository)?;
        let root_file = open_root(&root)?;
        let (parent, resolved_parent) =
            validated_source_parent(&repository, &root, &root_file, "harness/AGENTS.md")?;
        fs::rename(
            repository.join("harness"),
            repository.join("retained-harness"),
        )?;
        symlink(&outside, repository.join("harness"))?;

        let source = read_source_from_parent(
            &root,
            &root_file,
            &BTreeSet::new(),
            &parent,
            &resolved_parent,
            "harness/AGENTS.md".to_owned(),
            Category::Instructions,
        )?;
        assert_eq!(source.contents, "inside\n");
        Ok(())
    }

    #[test]
    fn replaced_canonical_parent_link_is_not_opened()
    -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let fixture = tempfile::tempdir()?;
        let repository = fixture.path().join("repository");
        let outside = fixture.path().join("outside");
        fs::create_dir_all(repository.join("harness/skills"))?;
        fs::create_dir(&outside)?;
        let root = fs::canonicalize(&repository)?;
        let root_file = open_root(&root)?;
        let relative = fs::canonicalize(repository.join("harness/skills"))?
            .strip_prefix(&root)?
            .to_owned();
        fs::rename(
            repository.join("harness/skills"),
            repository.join("retained-skills"),
        )?;
        symlink(&outside, repository.join("harness/skills"))?;

        assert!(open_directory(&root_file, &relative).is_err());
        Ok(())
    }

    #[test]
    fn replaced_leaf_link_is_not_opened() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let fixture = tempfile::tempdir()?;
        let repository = fixture.path().join("repository");
        let outside = fixture.path().join("outside");
        fs::create_dir_all(repository.join("harness"))?;
        fs::create_dir(&outside)?;
        fs::write(repository.join("harness/AGENTS.md"), "inside\n")?;
        fs::write(outside.join("AGENTS.md"), "outside\n")?;
        let root = fs::canonicalize(&repository)?;
        let root_file = open_root(&root)?;
        let parent = open_directory(&root_file, Path::new("harness"))?;
        fs::rename(
            repository.join("harness/AGENTS.md"),
            repository.join("harness/retained.md"),
        )?;
        symlink(
            outside.join("AGENTS.md"),
            repository.join("harness/AGENTS.md"),
        )?;

        assert!(open_regular_file(&parent, OsStr::new("AGENTS.md"), "harness/AGENTS.md").is_err());
        Ok(())
    }
}
