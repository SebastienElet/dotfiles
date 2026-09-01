use super::support::*;

#[test]
fn derives_the_id_from_nfc_and_collapsed_unicode_whitespace() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft(
        "Cafe\u{301}\t\n invariant",
        "café invariant",
        "Established.",
    );

    let id = stored_id(store.admit(resolved(&draft, &context), None, &timestamp, &context));

    assert_eq!(id, "mem_61194ae236ce5c2f28a4f8d6");
    assert!(root.join(format!("entries/user/{id}.yaml")).is_file());
}

#[test]
fn accepts_only_validated_utc_timestamps() {
    assert_eq!(
        parse_utc_timestamp("2026-08-28T12:00:00.123Z")
            .unwrap()
            .as_str(),
        "2026-08-28T12:00:00.123Z"
    );
    for invalid in ["", "2026-08-28 12:00:00Z", "2026-08-28T12:00:60Z"] {
        assert_eq!(
            parse_utc_timestamp(invalid).unwrap_err().code(),
            "invalid_field",
            "{invalid}"
        );
    }
}

#[test]
fn makes_identical_admission_idempotent_across_generated_timestamps() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let draft = user_draft(
        "A durable invariant remains independently useful.",
        "durable invariant",
        "The explicit decision establishes the invariant.",
    );
    let first_time = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let retry_time = parse_utc_timestamp("2026-08-28T12:05:00Z").unwrap();

    let first_id = stored_id(store.admit(resolved(&draft, &context), None, &first_time, &context));
    let retry = store.admit(resolved(&draft, &context), None, &retry_time, &context);

    match retry {
        AdmissionResult::Duplicate { id } => assert_eq!(id.as_str(), first_id),
        result => panic!("unexpected admission result: {result:?}"),
    }
    assert_eq!(store.list().unwrap().entries().len(), 1);
}

#[test]
fn reports_conflict_for_divergent_content_at_the_same_identity() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let statement = "A durable invariant remains independently useful.";
    let first = user_draft(statement, "first lookup", "Established.");
    let divergent = user_draft(statement, "different lookup", "Established.");
    stored_id(store.admit(resolved(&first, &context), None, &timestamp, &context));

    let result = store.admit(resolved(&divergent, &context), None, &timestamp, &context);

    assert_conflict(result, "entry_conflict");
    assert_eq!(store.list().unwrap().entries().len(), 1);
}

#[test]
fn rejects_scope_mismatch_before_writing() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let common = fixture.path().join("common.git");
    fs::create_dir(&common).unwrap();
    let scope_runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    let project = resolve_project(fixture.path(), &scope_runner).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let user = user_draft("User invariant.", "user invariant", "Established.");
    let project_draft = draft_yaml(
        "project",
        "Project invariant.",
        "project invariant",
        "Established.",
        "user-decision",
        "decision:project-invariant",
    );

    assert_rejected(
        store.admit(
            resolved(&user, &context),
            Some(&project),
            &timestamp,
            &context,
        ),
        "scope_mismatch",
    );
    assert_rejected(
        store.admit(
            resolved(&project_draft, &context),
            None,
            &timestamp,
            &context,
        ),
        "scope_mismatch",
    );
    assert!(store.list().unwrap().entries().is_empty());
}

#[test]
fn rechecks_local_sources_under_the_admission_lock() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let source = fixture.path().join("proof");
    fs::write(&source, b"before").unwrap();
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let draft = draft_yaml(
        "user",
        "The local proof remains stable.",
        "local proof",
        "The local file establishes the statement.",
        "local-file",
        source.to_str().unwrap(),
    );
    let resolved = resolved(&draft, &context);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    fs::write(&source, b"after").unwrap();

    let result = store.admit(resolved, None, &timestamp, &context);

    assert_conflict(result, "source_changed");
    assert!(store.list().unwrap().entries().is_empty());
}
