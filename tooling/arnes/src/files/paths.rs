use crate::Roots;
use crate::manifest::Scope;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct FileIdentity {
    device: u64,
    inode: u64,
}

pub fn canonical_within(path: &Path, root: &Path) -> Option<PathBuf> {
    let canonical_root = fs::canonicalize(root).ok()?;
    symlinks_within(path, root, &canonical_root, &mut Vec::new())
        .then(|| fs::canonicalize(path).ok())?
        .filter(|path| path.starts_with(canonical_root))
}

pub fn file_identity_within(path: &Path, root: &Path) -> Option<FileIdentity> {
    let canonical = canonical_within(path, root)?;
    let metadata = fs::metadata(canonical).ok()?;
    Some(FileIdentity {
        device: metadata.dev(),
        inode: metadata.ino(),
    })
}

fn symlinks_within(
    path: &Path,
    root: &Path,
    canonical_root: &Path,
    active: &mut Vec<PathBuf>,
) -> bool {
    let Some((base, relative)) = path
        .strip_prefix(root)
        .map(|relative| (root, relative))
        .or_else(|_| {
            path.strip_prefix(canonical_root)
                .map(|relative| (canonical_root, relative))
        })
        .ok()
    else {
        return route_within(path, canonical_root);
    };
    let mut candidate = base.to_path_buf();
    for component in relative.components() {
        candidate.push(component);
        let Ok(metadata) = fs::symlink_metadata(&candidate) else {
            return false;
        };
        if metadata.file_type().is_symlink() {
            if active.contains(&candidate) {
                return false;
            }
            let Ok(target) = fs::read_link(&candidate) else {
                return false;
            };
            let Some(target) = normalize(if target.is_absolute() {
                target
            } else {
                candidate.parent().unwrap_or(base).join(target)
            }) else {
                return false;
            };
            active.push(candidate.clone());
            let valid = symlinks_within(&target, root, canonical_root, active);
            active.pop();
            if !valid {
                return false;
            }
        }
    }
    true
}

fn route_within(path: &Path, canonical_root: &Path) -> bool {
    let mut candidate = PathBuf::new();
    for component in path.components() {
        candidate.push(component);
        let Ok(resolved) = fs::canonicalize(&candidate) else {
            return false;
        };
        if !canonical_root.starts_with(&resolved) && !resolved.starts_with(canonical_root) {
            return false;
        }
    }
    true
}

fn normalize(path: PathBuf) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir if normalized.pop() => {}
            Component::ParentDir => return None,
            _ => normalized.push(component.as_os_str()),
        }
    }
    Some(normalized)
}

pub fn ancestor_within(path: &Path, root: &Path) -> bool {
    let mut ancestor = path;
    while fs::symlink_metadata(ancestor).is_err_and(|error| error.kind() == ErrorKind::NotFound) {
        let Some(parent) = ancestor.parent() else {
            return false;
        };
        ancestor = parent;
    }
    canonical_within(ancestor, root).is_some()
}

pub fn planned_within(path: &Path, root: &Path) -> Option<PathBuf> {
    let mut ancestor = path;
    let mut suffix = Vec::new();
    while fs::symlink_metadata(ancestor).is_err_and(|error| error.kind() == ErrorKind::NotFound) {
        suffix.push(ancestor.file_name()?.to_owned());
        ancestor = ancestor.parent()?;
    }
    let mut identity = canonical_within(ancestor, root)?;
    for component in suffix.into_iter().rev() {
        identity.push(component);
    }
    Some(identity)
}

pub fn destination(roots: &Roots, scope: Scope, path: &Path) -> PathBuf {
    match scope {
        Scope::User => roots.home().join(path),
        Scope::Project => roots.repository().join(path),
    }
}

pub fn label(scope: Scope, path: &Path) -> String {
    match scope {
        Scope::User => format!("~/{}", path.display()),
        Scope::Project => path.display().to_string(),
    }
}

pub fn parent_within(path: &Path, root: &Path) -> bool {
    path.parent()
        .is_some_and(|parent| resolves_within(parent, root))
}

pub fn resolves_within(path: &Path, root: &Path) -> bool {
    canonical_within(path, root).is_some()
}

pub fn same_file(left: &Path, right: &Path) -> bool {
    match (fs::canonicalize(left), fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}
