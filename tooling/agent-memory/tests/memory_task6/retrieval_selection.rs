use super::support::*;
use agent_memory::{
    MemoryKind, OmissionEffect, ProofAnswers, ProofValid, RetrievalContext, RetrievalRequest,
    SourceKind, SourceSummary, Store, StoreFailpoint, retrieve,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Barrier};

#[test]
fn injects_only_reparsed_valid_entries_with_redacted_source_summaries_and_age() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let key = project_key(fixture.path());
    let yaml = project_entry_yaml(
        'd',
        "invariant",
        &key,
        &[
            SourceFixture {
                kind: "git-file",
                locator: "docs/contract.md",
                fingerprint: 'a',
            },
            SourceFixture {
                kind: "local-file",
                locator: "/Users/private/proof",
                fingerprint: 'b',
            },
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/rules?private=query#private-fragment",
                fingerprint: 'c',
            },
            SourceFixture {
                kind: "user-decision",
                locator: "decision:private-body",
                fingerprint: 'd',
            },
        ],
    );
    write_project_entry(&root, &key, 'd', &yaml);
    let selection = select(&store, &key, 5);
    let resolver = FakeResolver::with_responses([valid('a'), valid('b'), valid('c'), valid('d')]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");
    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment()),
    );

    assert!(report.omitted.is_empty());
    assert_eq!(report.injected.len(), 1);
    let injected = &report.injected[0];
    assert_eq!(injected.kind, MemoryKind::Invariant);
    assert_eq!(injected.statement, "Durable memory statement d.");
    assert_eq!(injected.verdict_age_milliseconds, 0);
    assert_eq!(
        injected.sources,
        [
            SourceSummary::with_locator(SourceKind::GitFile, "docs/contract.md"),
            SourceSummary::redacted(SourceKind::LocalFile),
            SourceSummary::with_locator(SourceKind::OfficialUrl, "https://docs.example.test/rules"),
            SourceSummary::redacted(SourceKind::UserDecision),
        ]
    );
    let diagnostic = format!("{:?}", report);
    assert!(!diagnostic.contains("/Users/private/proof"));
    assert!(!diagnostic.contains("decision:private-body"));
    assert!(!diagnostic.contains("private=query"));
    assert!(!diagnostic.contains("private-fragment"));
}

#[test]
fn preserves_search_limit_omissions_and_loads_no_unselected_yaml() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    for id in ['1', '2', '3', '4', '5', '6'] {
        let yaml = entry_yaml(
            id,
            "invariant",
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:limit",
                fingerprint: id,
            }],
        );
        write_user_entry(&root, id, &yaml);
    }
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 10);
    assert_eq!(selection.selected.len(), 5);
    assert_eq!(selection.omitted_by_limit, 1);
    let mut answers = ProofAnswers::new();
    for selected in &selection.selected {
        answers.insert(ProofValid::new(&selected.entry_id).unwrap());
    }
    let unselected = root.join("entries/user/mem_666666666666666666666666.yaml");
    fs::write(&unselected, b"unreadable unselected yaml").unwrap();
    fs::set_permissions(&unselected, fs::Permissions::from_mode(0o000)).unwrap();
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");
    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment())
            .with_proof_answers(&answers),
    );

    assert_eq!(report.injected.len(), 5);
    assert_eq!(report.omitted_by_limit, 1);
    fs::set_permissions(&unselected, fs::Permissions::from_mode(0o600)).unwrap();
}

#[test]
fn entry_changes_after_selection_are_omitted_without_old_context() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        '7',
        "invariant",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:before",
            fingerprint: '7',
        }],
    );
    let path = write_user_entry(&root, '7', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let changed = String::from_utf8(yaml).unwrap().replace(
        "Durable memory statement 7.",
        "Changed memory statement seven.",
    );
    fs::write(&path, changed).unwrap();
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");
    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment()),
    );

    assert!(report.injected.is_empty());
    assert_eq!(report.omitted[0].code, "selection_stale");
    assert_eq!(report.omitted[0].effect, OmissionEffect::NotApplied);
    let diagnostic = format!("{:?}", report.omitted[0]);
    assert!(!diagnostic.contains("Changed memory statement"));
    assert!(!diagnostic.contains("decision:before"));
}

#[test]
fn selection_identity_fields_are_revalidated_against_the_yaml() {
    for field in ["id", "kind", "path"] {
        let fixture = tempfile::tempdir().unwrap();
        let (root, store) = open_store(fixture.path());
        let yaml = entry_yaml(
            '8',
            "invariant",
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:identity",
                fingerprint: '8',
            }],
        );
        write_user_entry(&root, '8', &yaml);
        let key = project_key(fixture.path());
        let mut selection = select(&store, &key, 5);
        match field {
            "id" => selection.selected[0].entry_id = "mem_999999999999999999999999".to_owned(),
            "kind" => selection.selected[0].kind = "goal".to_owned(),
            "path" => {
                selection.selected[0].path =
                    "entries/user/mem_999999999999999999999999.yaml".to_owned()
            }
            _ => unreachable!(),
        }
        let resolver = FakeResolver::with_responses([]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        let report = retrieve(
            RetrievalRequest::new(&selection, &key, true),
            RetrievalContext::new(&store, &clock, &resolver, environment()),
        );

        assert!(report.injected.is_empty(), "{field}");
        assert_eq!(
            report.omitted[0].effect,
            OmissionEffect::NotApplied,
            "{field}"
        );
    }
}

#[test]
fn entry_substitution_during_reparse_is_omitted() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, initial) = open_store(fixture.path());
    let yaml = entry_yaml(
        '9',
        "invariant",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:race",
            fingerprint: '9',
        }],
    );
    let path = write_user_entry(&root, '9', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&initial, &key, 5);
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        memory_root(&root),
        StoreFailpoint::PauseAfterRetrievalEntryRead(Arc::clone(&barrier)),
    )
    .unwrap();
    let worker_barrier = Arc::clone(&barrier);
    let worker = std::thread::spawn(move || {
        let resolver = FakeResolver::with_responses([]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        retrieve(
            RetrievalRequest::new(&selection, &key, true),
            RetrievalContext::new(&store, &clock, &resolver, environment()),
        )
    });
    worker_barrier.wait();
    fs::rename(&path, root.join("entry-displaced")).unwrap();
    fs::write(&path, yaml).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    worker_barrier.wait();
    let report = worker.join().unwrap();

    assert!(report.injected.is_empty());
    assert_eq!(report.omitted[0].code, "selection_stale");
}
