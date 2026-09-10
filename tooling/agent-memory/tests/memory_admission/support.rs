pub use agent_memory::{
    AdmissionAuthorization, AdmissionContext, AdmissionResult, Clock, EntryScope, MemoryRoot,
    Store, StoreFailpoint, SystemProcessRunner, UtcTimestamp, admit, parse_utc_timestamp,
    prepare_admission,
};
pub use std::fs;
use std::path::Path;
use std::process::Command;
pub use std::sync::{Arc, Barrier};

#[derive(Clone)]
pub struct FixedClock(UtcTimestamp);

impl FixedClock {
    pub fn new() -> Result<Self, agent_memory::MemoryError> {
        Ok(Self(parse_utc_timestamp("2026-08-29T12:00:00Z")?))
    }
}

impl Clock for FixedClock {
    fn now(&self) -> UtcTimestamp {
        self.0.clone()
    }
}

pub fn context<'a>(
    store: &'a Store,
    cwd: &'a Path,
    clock: &'a dyn Clock,
    processes: &'a SystemProcessRunner,
    authorization: AdmissionAuthorization,
) -> AdmissionContext<'a> {
    AdmissionContext {
        store,
        cwd,
        clock,
        processes,
        authorization,
    }
}

pub fn draft(
    scope: Option<&str>,
    kind: &str,
    statement: &str,
    source_kind: &str,
    locator: &str,
) -> Vec<u8> {
    let scope = scope.map_or(String::new(), |value| format!("scope: {value}\n"));
    let automated = if source_kind == "user-decision" {
        ""
    } else {
        "  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n"
    };
    format!(
        "schema_version: 1\nkind: {kind}\nstatement: {statement}\n{scope}retrieval_terms:\n  - durable memory\nproof:\n  summary: Source-backed proof.\n  sources:\n    - kind: {source_kind}\n      locator: {locator}\noracle:\n{automated}  human_fallback:\n    question: Does the proof remain valid?\n    valid_when: The proof remains observable.\n  outcomes:\n    valid: The memory remains valid.\n    invalidated: The memory is invalidated.\n"
    )
    .into_bytes()
}

pub fn initialize_repository(directory: &Path) -> std::io::Result<()> {
    assert!(
        Command::new("git")
            .arg("init")
            .arg("-q")
            .arg(directory)
            .status()?
            .success()
    );
    fs::write(directory.join("proof.txt"), b"proof")?;
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(["add", "proof.txt"])
            .status()?
            .success()
    );
    Ok(())
}

pub fn stored_id(result: AdmissionResult, index_rebuild_required: bool) -> Result<String, String> {
    if let AdmissionResult::Stored {
        id,
        index_rebuild_required: actual,
    } = result
    {
        assert_eq!(actual, index_rebuild_required);
        Ok(id.as_str().to_owned())
    } else {
        Err(format!("unexpected result: {result:?}"))
    }
}

pub fn assert_rejected(result: &AdmissionResult, code: &str) {
    assert!(
        matches!(result, AdmissionResult::Rejected { error } if error.code() == code),
        "unexpected result: {result:?}"
    );
}

pub fn assert_conflict(result: &AdmissionResult, code: &str) {
    assert!(
        matches!(result, AdmissionResult::Conflict { error, .. } if error.code() == code),
        "unexpected result: {result:?}"
    );
}
