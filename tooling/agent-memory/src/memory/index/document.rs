mod diagnostics;

use super::inventory::{InventoryItem, inventory_digest};
use crate::memory::MemoryError;
use crate::memory::search::normalized_tokens;
use crate::memory::store::document::{StoredEntry, StoredScope};
use crate::memory::store::inventory::{valid_memory_id, valid_project_key};
pub(super) use diagnostics::{IndexDiagnostics, diagnostic_scope};
use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct IndexDocument {
    schema_version: u8,
    inventory_digest: String,
    pub(super) entries: Vec<IndexRow>,
    pub(super) diagnostics: IndexDiagnostics,
}

impl IndexDocument {
    pub(super) fn empty() -> Result<Self, MemoryError> {
        Self::with_inventory(Vec::new(), &[], IndexDiagnostics::default())
    }

    pub(super) fn with_inventory(
        entries: Vec<IndexRow>,
        inventory: &[InventoryItem],
        diagnostics: IndexDiagnostics,
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
            && self.diagnostics.valid()
            && self.entries.len() + self.diagnostics.len() == inventory.len()
            && inventory.iter().all(|item| self.represents(item))
    }

    fn represents(&self, item: &InventoryItem) -> bool {
        self.entries.iter().any(|row| row.path == item.path) || self.diagnostics.represents(item)
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
    pub(crate) length: u64,
    pub(crate) modified_ns: i64,
}

impl IndexRow {
    pub(super) fn new(entry: &StoredEntry, item: &InventoryItem) -> Self {
        let mut statement_tokens = normalized_tokens(&entry.statement);
        statement_tokens.sort();
        statement_tokens.dedup();
        Self {
            id: entry.id.clone(),
            kind: entry.kind.clone(),
            status: entry.status.clone(),
            scope: entry.scope.clone(),
            retrieval_terms: entry.retrieval_terms.clone(),
            statement_tokens,
            summary: entry.proof.summary.chars().take(160).collect(),
            path: item.path.clone(),
            length: item.length,
            modified_ns: item.modified_ns,
        }
    }

    fn valid_for(&self, inventory: &[InventoryItem]) -> bool {
        self.status == "active"
            && valid_memory_id(&self.id)
            && valid_kind(&self.kind)
            && self.valid_scope_and_path()
            && valid_retrieval_terms(&self.retrieval_terms)
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

fn valid_kind(kind: &str) -> bool {
    matches!(
        kind,
        "goal" | "decision" | "evidence" | "invariant" | "unknown" | "assumption"
    )
}

fn valid_retrieval_terms(terms: &[String]) -> bool {
    (1..=20).contains(&terms.len())
        && terms
            .iter()
            .all(|term| (1..=100).contains(&term.chars().count()))
}

fn valid_statement_tokens(tokens: &[String]) -> bool {
    tokens.windows(2).all(|tokens| tokens[0] < tokens[1])
        && tokens.iter().all(|token| {
            let normalized = normalized_tokens(token);
            normalized.len() == 1 && normalized[0] == *token
        })
}

const fn store_error() -> MemoryError {
    MemoryError::new("store_unavailable", "store")
}

const fn unsafe_path() -> MemoryError {
    MemoryError::new("unsafe_store_path", "store")
}
