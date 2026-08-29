use super::support::*;

#[test]
fn creating_a_project_scope_syncs_its_parent_before_publication() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open_with_failpoint(
        memory_root(&root),
        StoreFailpoint::AfterProjectDirectoryFsync,
    )
    .unwrap();
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
    let draft = draft_yaml(
        "project",
        "Project directory durability.",
        "project durability",
        "Established.",
        "user-decision",
        "decision:project-directory-durability",
    );

    let result = store.admit(
        resolved(&draft, &context),
        Some(&project),
        &timestamp,
        &context,
    );

    assert_rejected(result, "store_unavailable");
    let project_directory = root.join(format!("entries/project/{}", project.key().as_str()));
    assert!(project_directory.is_dir());
    assert_eq!(fs::read_dir(project_directory).unwrap().count(), 0);
    assert!(store.list().unwrap().entries().is_empty());
}

#[test]
fn interrupted_writes_never_publish_partial_yaml_or_a_forward_index() {
    let pre_yaml = [
        StoreFailpoint::BeforeYamlTemporaryCreate,
        StoreFailpoint::BeforeYamlWrite,
        StoreFailpoint::BeforeYamlFlush,
        StoreFailpoint::BeforeYamlFsync,
        StoreFailpoint::BeforeYamlRename,
        StoreFailpoint::BeforeIndexTemporaryCreate,
        StoreFailpoint::BeforeIndexWrite,
        StoreFailpoint::BeforeIndexFlush,
        StoreFailpoint::BeforeIndexFsync,
    ];
    let post_yaml = [
        StoreFailpoint::AfterYamlRename,
        StoreFailpoint::BeforeYamlDirectoryFsync,
        StoreFailpoint::BeforeIndexRename,
    ];

    for failpoint in pre_yaml {
        assert_interrupted_state(failpoint, false);
    }
    for failpoint in post_yaml {
        assert_interrupted_state(failpoint, true);
    }
}

fn assert_interrupted_state(failpoint: StoreFailpoint, yaml_committed: bool) {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let assert_temporary = matches!(failpoint, StoreFailpoint::BeforeYamlRename);
    let store = Store::open_with_failpoint(memory_root(&root), failpoint.clone()).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft(
        "A durable invariant remains independently useful.",
        "durable invariant",
        "Established.",
    );

    let result = store.admit(resolved(&draft, &context), None, &timestamp, &context);
    let reopened = Store::open(memory_root(&root)).unwrap();
    let listing = reopened.list().unwrap();
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();

    if yaml_committed {
        match result {
            AdmissionResult::Stored {
                index_rebuild_required: true,
                ..
            } => {}
            result => panic!("{failpoint:?}: {result:?}"),
        }
        assert_eq!(listing.entries().len(), 1, "{failpoint:?}");
        assert!(listing.index_rebuild_required(), "{failpoint:?}");
        assert!(index["entries"].as_array().unwrap().is_empty());
    } else {
        assert_rejected(result, "store_unavailable");
        assert!(listing.entries().is_empty(), "{failpoint:?}");
        assert!(!listing.index_rebuild_required(), "{failpoint:?}");
        assert!(index["entries"].as_array().unwrap().is_empty());
    }

    let mut temporary_count = 0;
    for entry in fs::read_dir(root.join("entries/user")).unwrap() {
        let entry = entry.unwrap();
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if name.contains(".tmp-") {
            temporary_count += 1;
            assert_eq!(private_mode(&entry.path()), 0o600, "{failpoint:?}");
        } else {
            let bytes = fs::read(entry.path()).unwrap();
            assert!(parse_entry(&bytes).is_ok(), "{failpoint:?}");
        }
    }
    if assert_temporary {
        assert!(temporary_count > 0);
    }
}

#[test]
fn replace_active_atomically_updates_yaml_and_index_once() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft(
        "Transition invariant.",
        "transition invariant",
        "Established.",
    );
    let id = stored_id(store.admit(resolved(&draft, &context), None, &timestamp, &context));
    let path = root.join(format!("entries/user/{id}.yaml"));
    let terminal_yaml = fs::read_to_string(&path).unwrap().replacen(
        "status: active",
        "status: invalidated",
        1,
    ) + "transition:\n  from: active\n  to: invalidated\n  at: 2026-08-28T13:00:00Z\n  verdict: invalid\n  reason: The proof changed.\n";
    let terminal = parse_entry(terminal_yaml.as_bytes()).unwrap();

    let commit = store.replace_active(&terminal).unwrap();

    assert!(!commit.index_rebuild_required());
    assert_eq!(
        store.load(&id).unwrap().unwrap().status(),
        Status::Invalidated
    );
    let listing = store.list().unwrap();
    assert!(!listing.index_rebuild_required());
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    assert!(index["entries"].as_array().unwrap().is_empty());
    assert_eq!(
        store.replace_active(&terminal).unwrap_err().code(),
        "entry_not_active"
    );
}

#[test]
fn replace_active_refuses_changes_to_immutable_entry_fields() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft("Immutable invariant.", "original term", "Established.");
    let id = stored_id(store.admit(resolved(&draft, &context), None, &timestamp, &context));
    let path = root.join(format!("entries/user/{id}.yaml"));
    let before = fs::read(&path).unwrap();
    let terminal_yaml = String::from_utf8(before.clone())
        .unwrap()
        .replacen("status: active", "status: invalidated", 1)
        .replacen("- original term", "- changed term", 1)
        + "transition:\n  from: active\n  to: invalidated\n  at: 2026-08-28T13:00:00Z\n  verdict: invalid\n  reason: The proof changed.\n";
    let terminal = parse_entry(terminal_yaml.as_bytes()).unwrap();

    let error = store.replace_active(&terminal).unwrap_err();

    assert_eq!(error.code(), "entry_conflict");
    assert_eq!(fs::read(path).unwrap(), before);
}

#[test]
fn replace_active_refuses_changes_to_original_timestamps() {
    for field in ["created_at", "established_at"] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        let git = FakeProcessRunner::default();
        let curl = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &git, &curl);
        let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
        let draft = user_draft("Timestamp invariant.", "timestamp", "Established.");
        let id = stored_id(store.admit(resolved(&draft, &context), None, &timestamp, &context));
        let path = root.join(format!("entries/user/{id}.yaml"));
        let before = fs::read(&path).unwrap();
        let terminal_yaml = String::from_utf8(before.clone())
            .unwrap()
            .replacen("status: active", "status: invalidated", 1)
            .replacen(
                &format!("{field}: 2026-08-28T12:00:00Z"),
                &format!("{field}: 2026-08-28T12:01:00Z"),
                1,
            )
            + "transition:\n  from: active\n  to: invalidated\n  at: 2026-08-28T13:00:00Z\n  verdict: invalid\n  reason: The proof changed.\n";
        let terminal = parse_entry(terminal_yaml.as_bytes()).unwrap();

        let error = store.replace_active(&terminal).unwrap_err();

        assert_eq!(error.code(), "entry_conflict", "{field}");
        assert_eq!(fs::read(path).unwrap(), before, "{field}");
    }
}
