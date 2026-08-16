use crate::Roots;
use crate::manifest::Scope;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn canonical_within(path: &Path, root: &Path) -> Option<PathBuf> {
    let root = fs::canonicalize(root).ok()?;
    let path = fs::canonicalize(path).ok()?;
    path.starts_with(&root).then_some(path)
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
