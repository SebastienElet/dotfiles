use super::paths::{ancestor_within, canonical_within, parent_within, same_file};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

mod parser;

use parser::imports;
pub use parser::{leading_imports, without_leading_imports};

pub struct Resolver {
    root: PathBuf,
    alias_root: PathBuf,
    aliases: HashMap<PathBuf, PathBuf>,
}

pub struct Graph {
    visited: HashSet<PathBuf>,
}

#[derive(Debug)]
pub enum IncludeError {
    Missing(PathBuf),
    MissingLink(PathBuf),
    Dangling(PathBuf),
    WrongLink(PathBuf),
    NotFile(PathBuf),
    Unreadable(PathBuf),
    Escapes(String),
    OutsideRoot(PathBuf),
    Cycle(PathBuf),
}

impl Resolver {
    pub fn new(root: &Path) -> Self {
        Self::with_aliases(root, root, [])
    }

    pub fn with_aliases(
        root: &Path,
        alias_root: &Path,
        aliases: impl IntoIterator<Item = (PathBuf, PathBuf)>,
    ) -> Self {
        Self {
            root: root.to_owned(),
            alias_root: alias_root.to_owned(),
            aliases: aliases.into_iter().collect(),
        }
    }

    pub fn walk(&self, path: &Path) -> Result<Graph, IncludeError> {
        let mut active = HashSet::new();
        let mut visited = HashSet::new();
        self.visit(path, &mut active, &mut visited)?;
        Ok(Graph { visited })
    }

    pub fn read(&self, path: &Path) -> Result<String, IncludeError> {
        self.load(path).map(|(_, contents)| contents)
    }

    pub fn resolve(&self, parent: &Path, include: &str) -> Result<PathBuf, IncludeError> {
        let mut path = parent.to_owned();
        for component in Path::new(include).components() {
            match component {
                Component::CurDir => {}
                Component::Normal(part) => path.push(part),
                Component::ParentDir if path != self.root => {
                    path.pop();
                }
                _ => return Err(IncludeError::Escapes(include.to_owned())),
            }
        }
        if path.starts_with(&self.root) {
            Ok(path)
        } else {
            Err(IncludeError::Escapes(include.to_owned()))
        }
    }

    fn visit(
        &self,
        path: &Path,
        active: &mut HashSet<PathBuf>,
        visited: &mut HashSet<PathBuf>,
    ) -> Result<(), IncludeError> {
        if visited.contains(path) {
            return Ok(());
        }
        let (identity, contents) = self.load(path)?;
        if !active.insert(identity.clone()) {
            return Err(IncludeError::Cycle(path.to_owned()));
        }
        for include in imports(&contents) {
            let included = self.resolve(path.parent().unwrap_or(&self.root), &include)?;
            self.visit(&included, active, visited)?;
        }
        active.remove(&identity);
        visited.insert(path.to_owned());
        Ok(())
    }

    fn load(&self, path: &Path) -> Result<(PathBuf, String), IncludeError> {
        if let Some(source) = self.aliases.get(path) {
            if !parent_within(path, &self.root) {
                return Err(IncludeError::OutsideRoot(path.to_owned()));
            }
            let metadata = fs::symlink_metadata(path).map_err(|error| match error.kind() {
                ErrorKind::NotFound => IncludeError::MissingLink(path.to_owned()),
                _ => IncludeError::Unreadable(path.to_owned()),
            })?;
            if !metadata.file_type().is_symlink() {
                return Err(IncludeError::WrongLink(path.to_owned()));
            }
            if !same_file(path, source) {
                return Err(IncludeError::WrongLink(path.to_owned()));
            }
            return load_regular(source, &self.alias_root);
        }
        load_regular(path, &self.root)
    }
}

impl Graph {
    pub fn contains(&self, path: &Path) -> bool {
        self.visited.contains(path)
    }

    pub fn paths(&self) -> impl Iterator<Item = &Path> {
        self.visited.iter().map(PathBuf::as_path)
    }
}

pub fn read_regular(path: &Path, root: &Path) -> Result<String, IncludeError> {
    load_regular(path, root).map(|(_, contents)| contents)
}

fn load_regular(path: &Path, root: &Path) -> Result<(PathBuf, String), IncludeError> {
    if !ancestor_within(path.parent().unwrap_or(root), root) {
        return Err(IncludeError::OutsideRoot(path.to_owned()));
    }
    let link_metadata = fs::symlink_metadata(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound => IncludeError::Missing(path.to_owned()),
        _ => IncludeError::Unreadable(path.to_owned()),
    })?;
    let metadata = fs::metadata(path).map_err(|error| match error.kind() {
        ErrorKind::NotFound if link_metadata.file_type().is_symlink() => {
            IncludeError::Dangling(path.to_owned())
        }
        ErrorKind::NotFound => IncludeError::Missing(path.to_owned()),
        _ => IncludeError::Unreadable(path.to_owned()),
    })?;
    if !metadata.is_file() {
        return Err(IncludeError::NotFile(path.to_owned()));
    }
    let identity =
        canonical_within(path, root).ok_or_else(|| IncludeError::OutsideRoot(path.to_owned()))?;
    let contents =
        fs::read_to_string(path).map_err(|_| IncludeError::Unreadable(path.to_owned()))?;
    Ok((identity, contents))
}
