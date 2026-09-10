pub use crate::memory_support::{FakeProcessRunner, FakeResponse};
pub use agent_memory::{
    AdmissionAuthorization, AdmissionResult, MemoryErrorClass, MemoryRoot, SourceContext, Status,
    Store, StoreFailpoint, SystemProcessRunner, parse_draft, parse_entry, parse_utc_timestamp,
    resolve_project, resolve_sources, validate_draft,
};
pub use std::fs;
pub use std::os::unix::fs::{PermissionsExt, symlink};
pub use std::path::Path;
pub use std::process::Command;
pub use std::sync::{Arc, Barrier};
pub use std::time::{Duration, Instant};

pub const ROOT_WORKER_OUTPUT: &str = "AGENT_MEMORY_ROOT_WORKER_OUTPUT";

pub fn private_mode(path: &Path) -> std::io::Result<u32> {
    Ok(fs::symlink_metadata(path)?.permissions().mode() & 0o777)
}

pub fn memory_root(path: &Path) -> Result<MemoryRoot, agent_memory::MemoryError> {
    MemoryRoot::new(path)
}

pub fn draft_yaml(
    scope: &str,
    statement: &str,
    retrieval_term: &str,
    summary: &str,
    source_kind: &str,
    locator: &str,
) -> Vec<u8> {
    let automated = if source_kind == "user-decision" {
        ""
    } else {
        "  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n"
    };
    format!(
        "schema_version: 1\nkind: invariant\nstatement: {}\nscope: {scope}\nretrieval_terms:\n  - {}\nproof:\n  summary: {}\n  sources:\n    - kind: {source_kind}\n      locator: {}\noracle:\n{automated}  human_fallback:\n    question: Does the proof still establish this invariant?\n    valid_when: The proof remains observable.\n  outcomes:\n    valid: The invariant remains established.\n    invalidated: The proof no longer establishes the invariant.\n",
        serde_json::Value::String(statement.to_owned()),
        serde_json::Value::String(retrieval_term.to_owned()),
        serde_json::Value::String(summary.to_owned()),
        serde_json::Value::String(locator.to_owned()),
    )
    .into_bytes()
}

pub fn user_draft(statement: &str, retrieval_term: &str, summary: &str) -> Vec<u8> {
    draft_yaml(
        "user",
        statement,
        retrieval_term,
        summary,
        "user-decision",
        "decision:durable-memory-test",
    )
}

pub fn resolved(
    bytes: &[u8],
    context: &SourceContext<'_>,
) -> Result<agent_memory::ResolvedDraft, agent_memory::MemoryError> {
    let draft = parse_draft(bytes)?;
    let validated = validate_draft(draft, AdmissionAuthorization::ExplicitRequest)?;
    resolve_sources(validated, context)
}

pub fn stored_id(result: AdmissionResult) -> Result<String, String> {
    if let AdmissionResult::Stored {
        id,
        index_rebuild_required: false,
    } = result
    {
        Ok(id.as_str().to_owned())
    } else {
        Err(format!("unexpected admission result: {result:?}"))
    }
}

pub fn assert_rejected(result: &AdmissionResult, expected_code: &str) {
    assert!(
        matches!(result, AdmissionResult::Rejected { error } if error.code() == expected_code),
        "unexpected admission result: {result:?}"
    );
}

pub fn assert_conflict(result: &AdmissionResult, expected_code: &str) {
    let expected_class = if expected_code == "store_lock_unavailable" {
        MemoryErrorClass::Unavailable
    } else {
        MemoryErrorClass::Conflict
    };
    assert!(
        matches!(result, AdmissionResult::Conflict { error, .. } if error.code() == expected_code && error.class() == expected_class),
        "unexpected admission result: {result:?}"
    );
}
