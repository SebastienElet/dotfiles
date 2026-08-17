use super::paths::{canonical_within, label};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Scope};
use std::collections::HashSet;
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn unmanaged(
    roots: &Roots,
    agent: Agent,
    scope: Scope,
    root: &Path,
    declared: &HashSet<PathBuf>,
) -> Vec<Diagnostic> {
    let base = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let directory = base.join(root);
    if fs::symlink_metadata(&directory).is_ok() && canonical_within(&directory, base).is_none() {
        return vec![root_error(agent, scope, root, &directory)];
    }
    let names = match installations(&directory) {
        Ok(names) => names,
        Err(_) if missing(&directory) => return Vec::new(),
        Err(reason) => return vec![read_error(agent, scope, root, reason)],
    };
    names
        .into_iter()
        .filter(|name| !declared.contains(&root.join(name)))
        .filter(|name| !super::external::is_claude_skills_plugin(agent, &directory.join(name)))
        .map(|name| classify(&directory, agent, scope, root, &name))
        .collect()
}

pub fn installations(directory: &Path) -> Result<Vec<PathBuf>, String> {
    let unreadable = || format!("directory {} could not be read", directory.display());
    let mut names = Vec::new();
    for entry in fs::read_dir(directory).map_err(|_| unreadable())? {
        let entry = entry.map_err(|_| unreadable())?;
        let kind = entry.file_type().map_err(|_| unreadable())?;
        if kind.is_dir() || kind.is_symlink() {
            names.push(PathBuf::from(entry.file_name()));
        }
    }
    names.sort();
    Ok(names)
}

fn classify(directory: &Path, agent: Agent, scope: Scope, root: &Path, name: &Path) -> Diagnostic {
    let path = directory.join(name);
    let broken = fs::symlink_metadata(&path)
        .is_ok_and(|metadata| metadata.file_type().is_symlink())
        && !fs::metadata(&path).is_ok_and(|metadata| metadata.is_dir());
    let classification = if broken { "broken" } else { "unmanaged" };
    Diagnostic::new(
        "skills",
        State::Unsupported,
        format!(
            "{classification} {agent} {scope} skill {} at {}/{} is unmanaged and not Arnes-owned",
            name.display(),
            label(scope, root),
            name.display()
        ),
    )
}

fn missing(path: &Path) -> bool {
    fs::symlink_metadata(path).is_err_and(|error| error.kind() == ErrorKind::NotFound)
}

fn root_error(agent: Agent, scope: Scope, root: &Path, directory: &Path) -> Diagnostic {
    Diagnostic::new(
        "skills",
        State::Error,
        format!(
            "broken {agent} {scope} skills root {}: directory {} escapes its scope root",
            label(scope, root),
            directory.display()
        ),
    )
}

fn read_error(agent: Agent, scope: Scope, root: &Path, reason: String) -> Diagnostic {
    Diagnostic::new(
        "skills",
        State::Error,
        format!(
            "broken {agent} {scope} skills root {}: {reason}",
            label(scope, root)
        ),
    )
}
