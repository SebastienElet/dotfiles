#![cfg(test)]

use crate::memory_support::{FakeProcessRunner, FakeResponse};
use agent_memory::{
    AdmissionAuthorization, MemoryErrorClass, MemoryRoot, SourceContext, Store, StoreFailpoint,
    parse_draft, resolve_sources, validate_draft,
};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn disappearance_and_symlink_replacement_are_source_conflicts()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for replacement in ["missing", "symlink"] {
        let fixture = tempfile::tempdir()?;
        let source = fixture.path().join("proof");
        fs::write(&source, b"initial proof")?;
        let runner = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &runner, &runner);
        let resolved = resolve_sources(local_draft(&source)?, &context)?;
        fs::remove_file(&source)?;
        if replacement == "symlink" {
            let target = fixture.path().join("target");
            fs::write(&target, b"replacement proof")?;
            symlink(target, &source)?;
        }

        let error = resolved
            .recheck_sources(&context)
            .err()
            .ok_or("expected operation failure")?;

        assert_eq!(error.code(), "source_changed", "{replacement}");
        assert_eq!(error.class(), MemoryErrorClass::Conflict, "{replacement}");
    }
    Ok(())
}

#[test]
fn transport_and_permission_failures_are_unavailable()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let runner = FakeProcessRunner::with_responses([
        FakeResponse::success(b"200\nhttps://docs.example.test/proof\n203.0.113.10\n".to_vec())
            .with_body(b"official proof".to_vec()),
        FakeResponse::failure(28, Vec::new()),
    ]);
    let context = SourceContext::new(fixture.path(), &runner, &runner)
        .with_temporary_directory(fixture.path());
    let resolved = resolve_sources(official_draft()?, &context)?;

    let timeout = resolved
        .recheck_sources(&context)
        .err()
        .ok_or("expected operation failure")?;
    assert_eq!(timeout.code(), "source_unavailable");
    assert_eq!(timeout.class(), MemoryErrorClass::Unavailable);

    let permissions = Store::open_with_failpoint(
        &MemoryRoot::new(fixture.path().join("store"))?,
        StoreFailpoint::BeforeModeRepair,
    )
    .err()
    .ok_or("expected operation failure")?;
    assert_eq!(permissions.code(), "store_permissions_unavailable");
    assert_eq!(permissions.class(), MemoryErrorClass::Unavailable);
    Ok(())
}

fn local_draft(
    path: &std::path::Path,
) -> Result<agent_memory::ValidatedDraft, Box<dyn std::error::Error + Send + Sync>> {
    validated_draft(
        "local-file",
        path.to_str().ok_or("non-UTF-8 fixture path")?,
        false,
    )
}

fn official_draft() -> Result<agent_memory::ValidatedDraft, Box<dyn std::error::Error + Send + Sync>>
{
    let yaml = draft_yaml("official-url", "https://docs.example.test/proof", true);
    Ok(validate_draft(
        parse_draft(yaml.as_bytes())?,
        AdmissionAuthorization::ExplicitRequest,
    )?)
}

fn validated_draft(
    kind: &str,
    locator: &str,
    add_decision: bool,
) -> Result<agent_memory::ValidatedDraft, Box<dyn std::error::Error + Send + Sync>> {
    let yaml = draft_yaml(kind, locator, add_decision);
    Ok(validate_draft(
        parse_draft(yaml.as_bytes())?,
        AdmissionAuthorization::ExplicitRequest,
    )?)
}

fn draft_yaml(kind: &str, locator: &str, add_decision: bool) -> String {
    let decision = add_decision.then_some(
        "    - kind: user-decision\n      locator: decision:official-domain:docs.example.test\n",
    );
    format!(
        "schema_version: 1\nkind: invariant\nstatement: Classified source errors remain stable.\nscope: user\nretrieval_terms:\n  - classified source errors\nproof:\n  summary: The source establishes the error classification.\n  sources:\n    - kind: {kind}\n      locator: {}\n{}oracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the source remain valid?\n    valid_when: The source remains observable.\n  outcomes:\n    valid: The source is unchanged.\n    invalidated: The source changed.\n",
        serde_json::Value::from(locator),
        decision.unwrap_or_default(),
    )
}
