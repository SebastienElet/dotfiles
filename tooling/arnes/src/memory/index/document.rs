use super::IndexDiagnostic;
use super::inventory::{InventoryItem, inventory_digest};
use crate::memory::MemoryError;
use crate::memory::path::ManagedPath;
use crate::memory::search::normalized_tokens;
use crate::memory::store::document::{StoredEntry, StoredScope};
use crate::memory::store::inventory::{valid_memory_id, valid_project_key};
use serde::{Deserialize, Serialize};
use std::fs::Metadata;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct IndexDocument {
    schema_version: u8,
    inventory_digest: String,
    pub(super) entries: Vec<IndexRow>,
    pub(super) diagnostics: Vec<IndexDiagnostic>,
}

impl IndexDocument {
    pub(super) fn empty() -> Result<Self, MemoryError> {
        Self::with_inventory(Vec::new(), &[], Vec::new())
    }

    pub(super) fn with_inventory(
        entries: Vec<IndexRow>,
        inventory: &[InventoryItem],
        diagnostics: Vec<IndexDiagnostic>,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            schema_version: 1,
            inventory_digest: inventory_digest(inventory)?,
            entries,
            diagnostics,
        })
    }

    pub(super) fn valid_for(&self, inventory: &[InventoryItem]) -> bool {
        self.schema_version == 1
            && inventory_digest(inventory).is_ok_and(|digest| digest == self.inventory_digest)
            && self
                .entries
                .windows(2)
                .all(|rows| rows[0].path < rows[1].path)
            && self.entries.iter().all(|row| row.valid_for(inventory))
            && self.diagnostics.windows(2).all(|diagnostics| {
                diagnostic_key(&diagnostics[0]) < diagnostic_key(&diagnostics[1])
            })
            && self.diagnostics.iter().all(valid_diagnostic)
            && self.entries.len() + self.diagnostics.len() == inventory.len()
            && inventory.iter().all(|item| self.represents(item))
    }

    fn represents(&self, item: &InventoryItem) -> bool {
        self.entries.iter().any(|row| row.path == item.path)
            || item
                .path
                .strip_suffix(".yaml")
                .and_then(|path| path.rsplit('/').next())
                .is_some_and(|id| {
                    self.diagnostics
                        .iter()
                        .any(|diagnostic| diagnostic.entry_id == id)
                })
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IndexRow {
    pub(crate) id: String,
    pub(crate) kind: String,
    status: String,
    pub(crate) scope: StoredScope,
    pub(crate) retrieval_terms: Vec<String>,
    pub(crate) statement_tokens: Vec<String>,
    pub(super) summary: String,
    pub(crate) path: String,
    length: u64,
    modified_ns: i64,
}

impl IndexRow {
    pub(super) fn new(
        entry: &StoredEntry,
        path: &ManagedPath,
        metadata: &Metadata,
    ) -> Result<Self, MemoryError> {
        let mut statement_tokens = normalized_tokens(&entry.statement);
        statement_tokens.sort();
        statement_tokens.dedup();
        Ok(Self {
            id: entry.id.clone(),
            kind: entry.kind.clone(),
            status: entry.status.clone(),
            scope: entry.scope.clone(),
            retrieval_terms: entry.retrieval_terms.clone(),
            statement_tokens,
            summary: entry.proof.summary.chars().take(160).collect(),
            path: relative_string(path.relative())?,
            length: metadata.len(),
            modified_ns: modified_ns(metadata)?,
        })
    }

    fn valid_for(&self, inventory: &[InventoryItem]) -> bool {
        self.status == "active"
            && valid_memory_id(&self.id)
            && valid_kind(&self.kind)
            && self.valid_scope_and_path()
            && !self.retrieval_terms.is_empty()
            && self.summary.chars().count() <= 160
            && valid_statement_tokens(&self.statement_tokens)
            && inventory.iter().any(|item| {
                item.path == self.path
                    && item.length == self.length
                    && item.modified_ns == self.modified_ns
            })
    }

    fn valid_scope_and_path(&self) -> bool {
        match &self.scope {
            StoredScope::User => self.path == format!("entries/user/{}.yaml", self.id),
            StoredScope::Project { key } => {
                valid_project_key(key)
                    && self.path == format!("entries/project/{key}/{}.yaml", self.id)
            }
        }
    }
}

pub(super) fn index_bytes(document: &IndexDocument) -> Result<Vec<u8>, MemoryError> {
    let mut bytes = serde_json::to_vec_pretty(document).map_err(|_| store_error())?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub(super) fn relative_string(path: &Path) -> Result<String, MemoryError> {
    path.to_str().map(str::to_owned).ok_or_else(unsafe_path)
}

fn modified_ns(metadata: &Metadata) -> Result<i64, MemoryError> {
    metadata
        .mtime()
        .checked_mul(1_000_000_000)
        .and_then(|seconds| seconds.checked_add(metadata.mtime_nsec()))
        .ok_or_else(store_error)
}

fn valid_kind(kind: &str) -> bool {
    matches!(
        kind,
        "goal" | "decision" | "evidence" | "invariant" | "unknown" | "assumption"
    )
}

fn valid_statement_tokens(tokens: &[String]) -> bool {
    tokens.windows(2).all(|tokens| tokens[0] < tokens[1])
        && tokens.iter().all(|token| {
            let normalized = normalized_tokens(token);
            normalized.len() == 1 && normalized[0] == *token
        })
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

fn diagnostic_key(diagnostic: &IndexDiagnostic) -> (&str, &str, &str) {
    (&diagnostic.entry_id, &diagnostic.check, &diagnostic.effect)
}

const fn store_error() -> MemoryError {
    MemoryError::new("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::new("unsafe_store_path", "store")
}
