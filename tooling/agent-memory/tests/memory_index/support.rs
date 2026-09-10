pub use crate::memory_support::{FakeProcessRunner, FakeResponse};
use agent_memory::AdmissionAuthorization;
pub use agent_memory::{
    AdmissionResult, MemoryRoot, ProjectScope, SourceContext, Store, parse_draft,
    parse_utc_timestamp, resolve_project, resolve_sources, validate_draft,
};
use std::fs;
use std::path::Path;

pub fn memory_root(path: &Path) -> Result<MemoryRoot, agent_memory::MemoryError> {
    MemoryRoot::new(path)
}

pub fn project_scope(
    directory: &Path,
    name: &str,
) -> Result<ProjectScope, Box<dyn std::error::Error + Send + Sync>> {
    let common = directory.join(name);
    fs::create_dir(&common)?;
    let runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    Ok(resolve_project(directory, &runner)?)
}

pub fn admit_user(
    store: &Store,
    directory: &Path,
    statement: &str,
    retrieval_terms: &[&str],
    summary: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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

pub fn admit_project(
    store: &Store,
    directory: &Path,
    project: &ProjectScope,
    statement: &str,
    retrieval_terms: &[&str],
    summary: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(directory, &runner, &runner);
    let bytes = draft(scope, statement, retrieval_terms, summary);
    let parsed = parse_draft(&bytes)?;
    let validated = validate_draft(parsed, AdmissionAuthorization::ExplicitRequest)?;
    let resolved = resolve_sources(validated, &context)?;
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let result = store.admit(&resolved, project, &timestamp, &context);
    if let AdmissionResult::Stored {
        id,
        index_rebuild_required: false,
    } = result
    {
        Ok(id.as_str().to_owned())
    } else {
        Err(format!("unexpected admission result: {result:?}").into())
    }
}

fn draft(scope: &str, statement: &str, retrieval_terms: &[&str], summary: &str) -> Vec<u8> {
    let mut terms = String::new();
    for term in retrieval_terms {
        terms.push_str("  - ");
        terms.push_str(&serde_json::Value::String((*term).to_owned()).to_string());
        terms.push('\n');
    }
    format!(
        "schema_version: 1\nkind: invariant\nstatement: {}\nscope: {scope}\nretrieval_terms:\n{terms}proof:\n  summary: {}\n  sources:\n    - kind: user-decision\n      locator: decision:index-test\noracle:\n  human_fallback:\n    question: Does the proof remain valid?\n    valid_when: The decision remains in force.\n  outcomes:\n    valid: The invariant remains established.\n    invalidated: The invariant no longer applies.\n",
        serde_json::Value::String(statement.to_owned()),
        serde_json::Value::String(summary.to_owned()),
    )
    .into_bytes()
}
