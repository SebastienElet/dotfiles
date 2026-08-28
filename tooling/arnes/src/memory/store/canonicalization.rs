use super::StoredScope;
use crate::memory::{MemoryError, MemoryKind, SourceKind, Status, TransitionVerdict};
use sha2::{Digest, Sha256};
use unicode_normalization::UnicodeNormalization;

pub(super) fn memory_id(
    kind: &str,
    scope: &StoredScope,
    statement: &str,
) -> Result<String, MemoryError> {
    let normalized_nfc = statement.nfc().collect::<String>();
    let normalized_statement = normalized_nfc
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let preimage = serde_json::to_vec(&(1_u8, kind, scope.identity(), normalized_statement))
        .map_err(|_| store_error())?;
    let digest = format!("{:x}", Sha256::digest(preimage));
    Ok(format!("mem_{}", &digest[..24]))
}

pub(super) fn kind_name(kind: MemoryKind) -> &'static str {
    match kind {
        MemoryKind::Goal => "goal",
        MemoryKind::Decision => "decision",
        MemoryKind::Evidence => "evidence",
        MemoryKind::Invariant => "invariant",
        MemoryKind::Unknown => "unknown",
        MemoryKind::Assumption => "assumption",
    }
}

pub(super) fn source_name(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::GitFile => "git-file",
        SourceKind::LocalFile => "local-file",
        SourceKind::OfficialUrl => "official-url",
        SourceKind::UserDecision => "user-decision",
    }
}

pub(super) fn status_name(status: Status) -> &'static str {
    match status {
        Status::Active => "active",
        Status::Achieved => "achieved",
        Status::Abandoned => "abandoned",
        Status::Superseded => "superseded",
        Status::Invalidated => "invalidated",
        Status::Resolved => "resolved",
        Status::Confirmed => "confirmed",
    }
}

pub(super) fn transition_verdict_name(verdict: TransitionVerdict) -> &'static str {
    match verdict {
        TransitionVerdict::Valid => "valid",
        TransitionVerdict::Invalid => "invalid",
    }
}

const fn store_error() -> MemoryError {
    MemoryError::new("store_unavailable", "store")
}
