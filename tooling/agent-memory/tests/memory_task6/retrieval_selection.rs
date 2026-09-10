use super::support::*;
use agent_memory::{
    MemoryKind, OmissionEffect, ProofAnswers, ProofValid, RetrievalContext, RetrievalRequest,
    SourceKind, SourceSummary, Store, StoreFailpoint, retrieve,
};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::sync::{Arc, Barrier};

#[test]
fn saturates_untrusted_selection_omission_counts()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (_, store) = open_store(fixture.path())?;
    let key = project_key(fixture.path())?;
    let selection = agent_memory::SearchSelection {
        selected: vec![
            agent_memory::SelectedMemory {
                entry_id: "mem_999999999999999999999999".to_owned(),
                kind: "invariant".to_owned(),
                path: "entries/user/mem_999999999999999999999999.yaml".to_owned(),
                length: 0,
                modified_ns: 0,
            };
            6
        ],
        omitted_by_limit: usize::MAX,
        diagnostics: Vec::new(),
    };
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;

    let report = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert_eq!(report.omitted_by_limit, usize::MAX);
    Ok(())
}

#[test]
fn injects_only_reparsed_valid_entries_with_redacted_source_summaries_and_age()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let key = project_key(fixture.path())?;
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
    )?;
    write_project_entry(&root, &key, 'd', &yaml)?;
    let selection = select(&store, &key, 5)?;
    let resolver = FakeResolver::with_responses([valid('a'), valid('b'), valid('c'), valid('d')]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let report = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert!(report.omitted.is_empty());
    assert_eq!(report.injected.len(), 1);
    let injected = &report.injected.first().ok_or("missing fixture element")?;
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
    let diagnostic = format!("{report:?}");
    assert!(!diagnostic.contains("/Users/private/proof"));
    assert!(!diagnostic.contains("decision:private-body"));
    assert!(!diagnostic.contains("private=query"));
    assert!(!diagnostic.contains("private-fragment"));
    Ok(())
}

#[test]
fn preserves_search_limit_omissions_and_loads_no_unselected_yaml()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    for id in ['1', '2', '3', '4', '5', '6'] {
        let yaml = entry_yaml(
            id,
            "invariant",
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:limit",
                fingerprint: id,
            }],
        )?;
        write_user_entry(&root, id, &yaml)?;
    }
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 10)?;
    assert_eq!(selection.selected.len(), 5);
    assert_eq!(selection.omitted_by_limit, 1);
    let mut answers = ProofAnswers::new();
    for selected in &selection.selected {
        answers.insert(ProofValid::new(&selected.entry_id)?);
    }
    let selected_ids = selection
        .selected
        .iter()
        .map(|selected| selected.entry_id.as_str())
        .collect::<Vec<_>>();
    let unselected_id = ['1', '2', '3', '4', '5', '6']
        .map(|id| user_entry_id(id, "invariant"))
        .into_iter()
        .find(|id| !selected_ids.contains(&id.as_str()))
        .ok_or("missing fixture value")?;
    let unselected = root.join(format!("entries/user/{unselected_id}.yaml"));
    fs::write(&unselected, b"unreadable unselected yaml")?;
    fs::set_permissions(&unselected, fs::Permissions::from_mode(0o000))?;
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let report = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment())
                .with_proof_answers(&answers),
        ))
    })?;

    assert_eq!(report.injected.len(), 5);
    assert_eq!(report.omitted_by_limit, 1);
    fs::set_permissions(&unselected, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[test]
fn entry_changes_after_selection_are_omitted_without_old_context()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        '7',
        "invariant",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:before",
            fingerprint: '7',
        }],
    )?;
    let path = write_user_entry(&root, '7', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 5)?;
    let changed = String::from_utf8(yaml)?.replace(
        "Durable memory statement 7.",
        "Changed memory statement seven.",
    );
    fs::write(&path, changed)?;
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let report = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert!(report.injected.is_empty());
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .code,
        "selection_stale"
    );
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .effect,
        OmissionEffect::NotApplied
    );
    let diagnostic = format!(
        "{:?}",
        report.omitted.first().ok_or("missing fixture element")?
    );
    assert!(!diagnostic.contains("Changed memory statement"));
    assert!(!diagnostic.contains("decision:before"));
    Ok(())
}

#[test]
fn selection_identity_fields_are_revalidated_against_the_yaml()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for field in ["id", "kind", "path"] {
        let fixture = tempfile::tempdir()?;
        let (root, store) = open_store(fixture.path())?;
        let yaml = entry_yaml(
            '8',
            "invariant",
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:identity",
                fingerprint: '8',
            }],
        )?;
        write_user_entry(&root, '8', &yaml)?;
        let key = project_key(fixture.path())?;
        let mut selection = select(&store, &key, 5)?;
        match field {
            "id" => {
                selection
                    .selected
                    .get_mut(0)
                    .ok_or("missing fixture element")?
                    .entry_id = "mem_999999999999999999999999".to_owned();
            }
            "kind" => {
                selection
                    .selected
                    .get_mut(0)
                    .ok_or("missing fixture element")?
                    .kind = "goal".to_owned();
            }
            "path" => {
                selection
                    .selected
                    .get_mut(0)
                    .ok_or("missing fixture element")?
                    .path = "entries/user/mem_999999999999999999999999.yaml".to_owned();
            }
            _ => return Err(format!("unknown fixture field: {field}").into()),
        }
        let resolver = FakeResolver::with_responses([]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
        let report = resolver.checked(|checked_resolver| {
            Ok(retrieve(
                RetrievalRequest::new(&selection, &key, true),
                &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
            ))
        })?;

        assert!(report.injected.is_empty(), "{field}");
        assert_eq!(
            report
                .omitted
                .first()
                .ok_or("missing fixture element")?
                .effect,
            OmissionEffect::NotApplied,
            "{field}"
        );
    }
    Ok(())
}

#[test]
fn entry_substitution_during_reparse_is_omitted()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, initial) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        '9',
        "invariant",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:race",
            fingerprint: '9',
        }],
    )?;
    let path = write_user_entry(&root, '9', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&initial, &key, 5)?;
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        &memory_root(&root)?,
        StoreFailpoint::PauseAfterRetrievalEntryRead(Arc::clone(&barrier)),
    )?;
    let worker_barrier = Arc::clone(&barrier);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let worker = std::thread::spawn(move || {
        let resolver = FakeResolver::with_responses([]);
        resolver.checked(|checked_resolver| {
            Ok(retrieve(
                RetrievalRequest::new(&selection, &key, true),
                &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
            ))
        })
    });
    worker_barrier.wait();
    fs::rename(&path, root.join("entry-displaced"))?;
    fs::write(&path, yaml)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    worker_barrier.wait();
    let report = worker.join().map_err(|_| "worker thread panicked")??;

    assert!(report.injected.is_empty());
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .code,
        "selection_stale"
    );
    Ok(())
}
