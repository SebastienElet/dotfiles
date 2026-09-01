pub(crate) use crate::memory_support::{FakeProcessRunner, FakeResponse};
use agent_memory::AdmissionAuthorization;
#[allow(unused_imports)]
pub(crate) use agent_memory::{
    AdmissionResult, Index, MemoryRoot, ProjectScope, SourceContext, Store, StoreFailpoint,
    parse_draft, parse_utc_timestamp, resolve_project, resolve_sources, validate_draft,
};
use std::fs;
use std::path::Path;

pub(crate) fn memory_root(path: &Path) -> MemoryRoot {
    MemoryRoot::new(path).unwrap()
}

pub(crate) fn project_scope(directory: &Path, name: &str) -> ProjectScope {
    let common = directory.join(name);
    fs::create_dir(&common).unwrap();
    let runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    resolve_project(directory, &runner).unwrap()
}

pub(crate) fn admit_user(
    store: &Store,
    directory: &Path,
    statement: &str,
    retrieval_terms: &[&str],
    summary: &str,
) -> String {
    admit(
        store,
        directory,
        None,
        "user",
        statement,
        retrieval_terms,
        summary,
    )
}

pub(crate) fn admit_project(
    store: &Store,
    directory: &Path,
    project: &ProjectScope,
    statement: &str,
    retrieval_terms: &[&str],
    summary: &str,
) -> String {
    admit(
        store,
        directory,
        Some(project),
        "project",
        statement,
        retrieval_terms,
        summary,
    )
}

fn admit(
    store: &Store,
    directory: &Path,
    project: Option<&ProjectScope>,
    scope: &str,
    statement: &str,
    retrieval_terms: &[&str],
    summary: &str,
) -> String {
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(directory, &runner, &runner);
    let bytes = draft(scope, statement, retrieval_terms, summary);
    let parsed = parse_draft(&bytes).unwrap();
    let validated = validate_draft(parsed, AdmissionAuthorization::ExplicitRequest).unwrap();
    let resolved = resolve_sources(validated, &context).unwrap();
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    match store.admit(resolved, project, &timestamp, &context) {
        AdmissionResult::Stored {
            id,
            index_rebuild_required: false,
        } => id.as_str().to_owned(),
        result => panic!("unexpected admission result: {result:?}"),
    }
}

fn draft(scope: &str, statement: &str, retrieval_terms: &[&str], summary: &str) -> Vec<u8> {
    let terms = retrieval_terms
        .iter()
        .map(|term| format!("  - {}\n", serde_json::to_string(term).unwrap()))
        .collect::<String>();
    format!(
        "schema_version: 1\nkind: invariant\nstatement: {}\nscope: {scope}\nretrieval_terms:\n{terms}proof:\n  summary: {}\n  sources:\n    - kind: user-decision\n      locator: decision:index-test\noracle:\n  human_fallback:\n    question: Does the proof remain valid?\n    valid_when: The decision remains in force.\n  outcomes:\n    valid: The invariant remains established.\n    invalidated: The invariant no longer applies.\n",
        serde_json::to_string(statement).unwrap(),
        serde_json::to_string(summary).unwrap(),
    )
    .into_bytes()
}
