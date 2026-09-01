use super::super::IndexDiagnostic;
use super::InventoryItem;
use crate::memory::MemoryError;
use crate::memory::store::document::StoredScope;
use crate::memory::store::inventory::{valid_memory_id, valid_project_key};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(in crate::memory::index) struct IndexDiagnostics {
    user: Vec<IndexDiagnostic>,
    projects: BTreeMap<String, Vec<IndexDiagnostic>>,
}

impl IndexDiagnostics {
    pub(in crate::memory::index) fn push(
        &mut self,
        scope: &StoredScope,
        diagnostic: IndexDiagnostic,
    ) {
        match scope {
            StoredScope::User => self.user.push(diagnostic),
            StoredScope::Project { key } => {
                self.projects
                    .entry(key.clone())
                    .or_default()
                    .push(diagnostic);
            }
        }
    }

    pub(in crate::memory::index) fn sort(&mut self) {
        sort_diagnostics(&mut self.user);
        self.projects
            .values_mut()
            .for_each(|items| sort_diagnostics(items));
    }

    pub(in crate::memory::index) fn all(&self) -> Vec<IndexDiagnostic> {
        let mut diagnostics = self.user.clone();
        diagnostics.extend(self.projects.values().flatten().cloned());
        sort_diagnostics(&mut diagnostics);
        diagnostics
    }

    pub(in crate::memory::index) fn for_scope(
        &self,
        project_key: &str,
        include_user: bool,
    ) -> Vec<IndexDiagnostic> {
        let mut diagnostics = self.projects.get(project_key).cloned().unwrap_or_default();
        if include_user {
            diagnostics.extend(self.user.clone());
        }
        sort_diagnostics(&mut diagnostics);
        diagnostics
    }

    pub(in crate::memory::index) fn valid(&self) -> bool {
        valid_diagnostics(&self.user)
            && self.projects.iter().all(|(key, diagnostics)| {
                valid_project_key(key) && !diagnostics.is_empty() && valid_diagnostics(diagnostics)
            })
    }

    pub(in crate::memory::index) fn len(&self) -> usize {
        self.user.len() + self.projects.values().map(Vec::len).sum::<usize>()
    }

    pub(in crate::memory::index) fn represents(&self, item: &InventoryItem) -> bool {
        let Some((scope, id)) = diagnostic_location(&item.path) else {
            return false;
        };
        match scope {
            StoredScope::User => self.user.iter().any(|diagnostic| diagnostic.entry_id == id),
            StoredScope::Project { key } => self.projects.get(&key).is_some_and(|diagnostics| {
                diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.entry_id == id)
            }),
        }
    }
}

pub(in crate::memory::index) fn diagnostic_scope(path: &Path) -> Result<StoredScope, MemoryError> {
    let components = path
        .iter()
        .map(|part| part.to_str().ok_or_else(unsafe_path))
        .collect::<Result<Vec<_>, _>>()?;
    match components.as_slice() {
        ["entries", "user", _] => Ok(StoredScope::User),
        ["entries", "project", key, _] if valid_project_key(key) => Ok(StoredScope::Project {
            key: (*key).to_owned(),
        }),
        _ => Err(unsafe_path()),
    }
}

fn diagnostic_location(path: &str) -> Option<(StoredScope, &str)> {
    let parts = path.split('/').collect::<Vec<_>>();
    match parts.as_slice() {
        ["entries", "user", filename] => Some((StoredScope::User, filename.strip_suffix(".yaml")?)),
        ["entries", "project", key, filename] if valid_project_key(key) => Some((
            StoredScope::Project {
                key: (*key).to_owned(),
            },
            filename.strip_suffix(".yaml")?,
        )),
        _ => None,
    }
}

fn valid_diagnostics(diagnostics: &[IndexDiagnostic]) -> bool {
    diagnostics
        .windows(2)
        .all(|items| diagnostic_key(&items[0]) < diagnostic_key(&items[1]))
        && diagnostics.iter().all(valid_diagnostic)
}

fn valid_diagnostic(diagnostic: &IndexDiagnostic) -> bool {
    valid_memory_id(&diagnostic.entry_id)
        && diagnostic.effect == "omitted"
        && !diagnostic.check.is_empty()
        && diagnostic.check.len() <= 64
        && diagnostic
            .check
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
}

fn sort_diagnostics(diagnostics: &mut [IndexDiagnostic]) {
    diagnostics.sort_by(|left, right| diagnostic_key(left).cmp(&diagnostic_key(right)));
}

fn diagnostic_key(diagnostic: &IndexDiagnostic) -> (&str, &str, &str) {
    (&diagnostic.entry_id, &diagnostic.check, &diagnostic.effect)
}

const fn unsafe_path() -> MemoryError {
    MemoryError::unavailable("unsafe_store_path", "store")
}
