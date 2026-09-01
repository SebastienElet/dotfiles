use super::canonicalization::{
    kind_name, memory_id, source_name, status_name, transition_verdict_name,
};
use crate::memory::{
    EntryScope, MemoryEntry, MemoryError, ProjectScope, ResolvedDraft, ScopeDraft, UtcTimestamp,
    ValidatedOracle,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize)]
pub(crate) struct StoredEntry {
    schema_version: u8,
    pub(crate) id: String,
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) statement: String,
    pub(crate) scope: StoredScope,
    pub(crate) retrieval_terms: Vec<String>,
    pub(crate) proof: StoredProof,
    oracle: StoredOracle,
    created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    transition: Option<StoredTransition>,
}

impl StoredEntry {
    pub(super) fn from_resolved(
        resolved: &ResolvedDraft,
        project: Option<&ProjectScope>,
        timestamp: &UtcTimestamp,
    ) -> Result<Self, MemoryError> {
        let draft = resolved.draft();
        let scope = match (draft.scope(), project) {
            (ScopeDraft::User, None) => StoredScope::User,
            (ScopeDraft::Project, Some(project)) => StoredScope::Project {
                key: project.key().as_str().to_owned(),
            },
            _ => return Err(MemoryError::new("scope_mismatch", "scope")),
        };
        let kind = kind_name(draft.kind()).to_owned();
        let id = memory_id(&kind, &scope, draft.statement().as_str())?;
        Ok(Self {
            schema_version: 1,
            id,
            kind,
            status: "active".to_owned(),
            statement: draft.statement().as_str().to_owned(),
            scope,
            retrieval_terms: draft
                .retrieval_terms()
                .iter()
                .map(|term| term.as_str().to_owned())
                .collect(),
            proof: StoredProof {
                summary: draft.proof().summary().to_owned(),
                sources: resolved
                    .sources()
                    .iter()
                    .map(|source| StoredSource {
                        kind: source_name(source.kind()).to_owned(),
                        locator: source.locator().to_owned(),
                        fingerprint: source.fingerprint().as_str().to_owned(),
                    })
                    .collect(),
                established_at: timestamp.as_str().to_owned(),
            },
            oracle: StoredOracle::new(draft.oracle()),
            created_at: timestamp.as_str().to_owned(),
            transition: None,
        })
    }

    pub(crate) fn from_entry(entry: &MemoryEntry) -> Self {
        Self {
            schema_version: 1,
            id: entry.id().as_str().to_owned(),
            kind: kind_name(entry.kind()).to_owned(),
            status: status_name(entry.status()).to_owned(),
            statement: entry.statement().as_str().to_owned(),
            scope: StoredScope::from_entry(entry.scope()),
            retrieval_terms: entry
                .retrieval_terms()
                .iter()
                .map(|term| term.as_str().to_owned())
                .collect(),
            proof: StoredProof {
                summary: entry.proof().summary().to_owned(),
                sources: entry
                    .proof()
                    .sources()
                    .iter()
                    .map(|source| StoredSource {
                        kind: source_name(source.kind()).to_owned(),
                        locator: source.locator().to_owned(),
                        fingerprint: source.fingerprint().as_str().to_owned(),
                    })
                    .collect(),
                established_at: entry.proof().established_at().as_str().to_owned(),
            },
            oracle: StoredOracle::new(entry.oracle()),
            created_at: entry.created_at().as_str().to_owned(),
            transition: entry.transition().map(|transition| StoredTransition {
                from: status_name(transition.from()).to_owned(),
                to: status_name(transition.to()).to_owned(),
                at: transition.at().as_str().to_owned(),
                verdict: transition_verdict_name(transition.verdict()).to_owned(),
                reason: transition.reason().to_owned(),
            }),
        }
    }

    pub(super) fn canonical_value(&self) -> Result<serde_json::Value, MemoryError> {
        let mut value = serde_json::to_value(self).map_err(|_| store_error())?;
        let root = value.as_object_mut().ok_or_else(store_error)?;
        root.remove("created_at");
        let proof = root
            .get_mut("proof")
            .and_then(serde_json::Value::as_object_mut)
            .ok_or_else(store_error)?;
        proof.remove("established_at");
        Ok(value)
    }

    pub(super) fn immutable_value(&self) -> Result<serde_json::Value, MemoryError> {
        let mut value = serde_json::to_value(self).map_err(|_| store_error())?;
        let root = value.as_object_mut().ok_or_else(store_error)?;
        root.remove("status");
        root.remove("transition");
        Ok(value)
    }

    pub(super) fn validate_identity(&self) -> Result<(), MemoryError> {
        if memory_id(&self.kind, &self.scope, &self.statement)? == self.id {
            Ok(())
        } else {
            Err(MemoryError::unavailable("entry_identity_mismatch", "id"))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase", deny_unknown_fields)]
pub(crate) enum StoredScope {
    Project { key: String },
    User,
}

impl StoredScope {
    fn from_entry(scope: &EntryScope) -> Self {
        match scope {
            EntryScope::Project(key) => Self::Project {
                key: key.as_str().to_owned(),
            },
            EntryScope::User => Self::User,
        }
    }

    pub(super) fn identity(&self) -> &str {
        match self {
            Self::Project { key } => key,
            Self::User => "user",
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct StoredProof {
    pub(crate) summary: String,
    sources: Vec<StoredSource>,
    established_at: String,
}

#[derive(Clone, Debug, Serialize)]
struct StoredSource {
    kind: String,
    locator: String,
    fingerprint: String,
}

#[derive(Clone, Debug, Serialize)]
struct StoredOracle {
    #[serde(skip_serializing_if = "Option::is_none")]
    automated: Option<StoredAutomatedOracle>,
    human_fallback: StoredHumanFallback,
    outcomes: StoredOutcomes,
}

impl StoredOracle {
    fn new(oracle: &ValidatedOracle) -> Self {
        Self {
            automated: oracle
                .has_automated_oracle()
                .then_some(StoredAutomatedOracle {
                    kind: "source-fingerprint".to_owned(),
                    expected: "all-proof-sources-unchanged".to_owned(),
                }),
            human_fallback: StoredHumanFallback {
                question: oracle.fallback_question().to_owned(),
                valid_when: oracle.fallback_valid_when().to_owned(),
            },
            outcomes: StoredOutcomes {
                valid: oracle.valid_outcome().to_owned(),
                invalidated: oracle.invalidated_outcome().to_owned(),
            },
        }
    }
}

#[derive(Clone, Debug, Serialize)]
struct StoredAutomatedOracle {
    kind: String,
    expected: String,
}

#[derive(Clone, Debug, Serialize)]
struct StoredHumanFallback {
    question: String,
    valid_when: String,
}

#[derive(Clone, Debug, Serialize)]
struct StoredOutcomes {
    valid: String,
    invalidated: String,
}

#[derive(Clone, Debug, Serialize)]
struct StoredTransition {
    from: String,
    to: String,
    at: String,
    verdict: String,
    reason: String,
}

pub(super) fn yaml_bytes(entry: &StoredEntry) -> Result<Vec<u8>, MemoryError> {
    let mut bytes = serde_yaml_ng::to_string(entry)
        .map_err(|_| store_error())?
        .into_bytes();
    if !bytes.ends_with(b"\n") {
        bytes.push(b'\n');
    }
    Ok(bytes)
}

const fn store_error() -> MemoryError {
    MemoryError::unavailable("store_unavailable", "store")
}
