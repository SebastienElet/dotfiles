#[allow(dead_code)]
#[path = "support/memory.rs"]
mod memory_support;

use agent_memory::{
    AdmissionAuthorization, MemoryErrorClass, MemoryRoot, SourceContext, Store, StoreFailpoint,
    parse_draft, resolve_sources, validate_draft,
};
use memory_support::{FakeProcessRunner, FakeResponse};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn disappearance_and_symlink_replacement_are_source_conflicts() {
    for replacement in ["missing", "symlink"] {
        let fixture = tempfile::tempdir().unwrap();
        let source = fixture.path().join("proof");
        fs::write(&source, b"initial proof").unwrap();
        let runner = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &runner, &runner);
        let resolved = resolve_sources(local_draft(&source), &context).unwrap();
        fs::remove_file(&source).unwrap();
        if replacement == "symlink" {
            let target = fixture.path().join("target");
            fs::write(&target, b"replacement proof").unwrap();
            symlink(target, &source).unwrap();
        }

        let error = resolved.recheck_sources(&context).unwrap_err();

        assert_eq!(error.code(), "source_changed", "{replacement}");
        assert_eq!(error.class(), MemoryErrorClass::Conflict, "{replacement}");
    }
}

#[test]
fn transport_and_permission_failures_are_unavailable() {
    let fixture = tempfile::tempdir().unwrap();
    let runner = FakeProcessRunner::with_responses([
        FakeResponse::success(b"200\nhttps://docs.example.test/proof\n203.0.113.10\n".to_vec())
            .with_body(b"official proof".to_vec()),
        FakeResponse::failure(28, Vec::new()),
    ]);
    let context = SourceContext::new(fixture.path(), &runner, &runner)
        .with_temporary_directory(fixture.path());
    let resolved = resolve_sources(official_draft(), &context).unwrap();

    let timeout = resolved.recheck_sources(&context).unwrap_err();
    assert_eq!(timeout.code(), "source_unavailable");
    assert_eq!(timeout.class(), MemoryErrorClass::Unavailable);

    let permissions = Store::open_with_failpoint(
        MemoryRoot::new(fixture.path().join("store")).unwrap(),
        StoreFailpoint::BeforeModeRepair,
    )
    .unwrap_err();
    assert_eq!(permissions.code(), "store_permissions_unavailable");
    assert_eq!(permissions.class(), MemoryErrorClass::Unavailable);
}

fn local_draft(path: &std::path::Path) -> agent_memory::ValidatedDraft {
    validated_draft("local-file", path.to_str().unwrap(), false)
}

fn official_draft() -> agent_memory::ValidatedDraft {
    let yaml = draft_yaml("official-url", "https://docs.example.test/proof", true);
    validate_draft(
        parse_draft(yaml.as_bytes()).unwrap(),
        AdmissionAuthorization::ExplicitRequest,
    )
    .unwrap()
}

fn validated_draft(kind: &str, locator: &str, add_decision: bool) -> agent_memory::ValidatedDraft {
    let yaml = draft_yaml(kind, locator, add_decision);
    validate_draft(
        parse_draft(yaml.as_bytes()).unwrap(),
        AdmissionAuthorization::ExplicitRequest,
    )
    .unwrap()
}

fn draft_yaml(kind: &str, locator: &str, add_decision: bool) -> String {
    let decision = add_decision.then_some(
        "    - kind: user-decision\n      locator: decision:official-domain:docs.example.test\n",
    );
    format!(
        "schema_version: 1\nkind: invariant\nstatement: Classified source errors remain stable.\nscope: user\nretrieval_terms:\n  - classified source errors\nproof:\n  summary: The source establishes the error classification.\n  sources:\n    - kind: {kind}\n      locator: {}\n{}oracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the source remain valid?\n    valid_when: The source remains observable.\n  outcomes:\n    valid: The source is unchanged.\n    invalidated: The source changed.\n",
        serde_json::to_string(locator).unwrap(),
        decision.unwrap_or_default(),
    )
}
