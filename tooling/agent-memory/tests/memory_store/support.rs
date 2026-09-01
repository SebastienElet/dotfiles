pub(crate) use crate::memory_support::{FakeProcessRunner, FakeResponse};
pub(crate) use agent_memory::{
    AdmissionAuthorization, AdmissionResult, MemoryErrorClass, MemoryRoot, SourceContext, Status,
    Store, StoreFailpoint, SystemProcessRunner, parse_draft, parse_entry, parse_utc_timestamp,
    resolve_project, resolve_sources, validate_draft,
};
pub(crate) use std::fs;
pub(crate) use std::os::unix::fs::{PermissionsExt, symlink};
pub(crate) use std::path::Path;
pub(crate) use std::process::Command;
pub(crate) use std::sync::{Arc, Barrier};
pub(crate) use std::time::{Duration, Instant};

pub(crate) const ROOT_WORKER_OUTPUT: &str = "AGENT_MEMORY_ROOT_WORKER_OUTPUT";

pub(crate) fn private_mode(path: &Path) -> u32 {
    fs::symlink_metadata(path).unwrap().permissions().mode() & 0o777
}

pub(crate) fn memory_root(path: &Path) -> MemoryRoot {
    MemoryRoot::new(path).unwrap()
}

pub(crate) fn draft_yaml(
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
        serde_json::to_string(statement).unwrap(),
        serde_json::to_string(retrieval_term).unwrap(),
        serde_json::to_string(summary).unwrap(),
        serde_json::to_string(locator).unwrap(),
    )
    .into_bytes()
}

pub(crate) fn user_draft(statement: &str, retrieval_term: &str, summary: &str) -> Vec<u8> {
    draft_yaml(
        "user",
        statement,
        retrieval_term,
        summary,
        "user-decision",
        "decision:durable-memory-test",
    )
}

pub(crate) fn resolved(bytes: &[u8], context: &SourceContext<'_>) -> agent_memory::ResolvedDraft {
    let draft = parse_draft(bytes).unwrap();
    let validated = validate_draft(draft, AdmissionAuthorization::ExplicitRequest).unwrap();
    resolve_sources(validated, context).unwrap()
}

pub(crate) fn stored_id(result: AdmissionResult) -> String {
    match result {
        AdmissionResult::Stored {
            id,
            index_rebuild_required: false,
        } => id.as_str().to_owned(),
        result => panic!("unexpected admission result: {result:?}"),
    }
}

pub(crate) fn assert_rejected(result: AdmissionResult, expected_code: &str) {
    match result {
        AdmissionResult::Rejected { error } => assert_eq!(error.code(), expected_code),
        result => panic!("unexpected admission result: {result:?}"),
    }
}

pub(crate) fn assert_conflict(result: AdmissionResult, expected_code: &str) {
    match result {
        AdmissionResult::Conflict { error, .. } => {
            assert_eq!(error.code(), expected_code);
            let expected_class = if expected_code == "store_lock_unavailable" {
                MemoryErrorClass::Unavailable
            } else {
                MemoryErrorClass::Conflict
            };
            assert_eq!(error.class(), expected_class);
        }
        result => panic!("unexpected admission result: {result:?}"),
    }
}
