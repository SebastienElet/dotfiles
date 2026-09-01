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
    let undurable_yaml = [
        StoreFailpoint::AfterYamlRename,
        StoreFailpoint::BeforeYamlDirectoryFsync,
    ];
    let durable_yaml = [StoreFailpoint::BeforeIndexRename];

    for failpoint in pre_yaml {
        assert_interrupted_state(failpoint, InterruptedAdmission::Absent);
    }
    for failpoint in undurable_yaml {
        assert_interrupted_state(failpoint, InterruptedAdmission::Renamed);
    }
    for failpoint in durable_yaml {
        assert_interrupted_state(failpoint, InterruptedAdmission::Durable);
    }
}

#[test]
fn retry_syncs_a_renamed_admission_before_reporting_duplicate() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft(
        "A retried admission becomes durable before success.",
        "retry durability",
        "Established.",
    );
    let interrupted =
        Store::open_with_failpoint(memory_root(&root), StoreFailpoint::AfterYamlRename).unwrap();
    assert_rejected(
        interrupted.admit(resolved(&draft, &context), None, &timestamp, &context),
        "store_unavailable",
    );

    let failed_retry =
        Store::open_with_failpoint(memory_root(&root), StoreFailpoint::BeforeYamlDirectoryFsync)
            .unwrap();
    assert_rejected(
        failed_retry.admit(resolved(&draft, &context), None, &timestamp, &context),
        "store_unavailable",
    );

    let durable_retry = Store::open(memory_root(&root)).unwrap();
    match durable_retry.admit(resolved(&draft, &context), None, &timestamp, &context) {
        AdmissionResult::Duplicate { .. } => {}
        result => panic!("unexpected admission result: {result:?}"),
    }
}

enum InterruptedAdmission {
    Absent,
    Renamed,
    Durable,
}

fn assert_interrupted_state(failpoint: StoreFailpoint, expected: InterruptedAdmission) {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open_with_failpoint(memory_root(&root), failpoint.clone()).unwrap();
    let before = directory_inventory(&root);
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

    match expected {
        InterruptedAdmission::Absent => {
            assert_rejected(result, "store_unavailable");
            assert!(listing.entries().is_empty(), "{failpoint:?}");
            assert!(!listing.index_rebuild_required(), "{failpoint:?}");
            assert!(index["entries"].as_array().unwrap().is_empty());
            assert_eq!(directory_inventory(&root), before, "{failpoint:?}");
        }
        InterruptedAdmission::Renamed => {
            assert_rejected(result, "store_unavailable");
            assert_eq!(listing.entries().len(), 1, "{failpoint:?}");
            assert!(listing.index_rebuild_required(), "{failpoint:?}");
            assert!(index["entries"].as_array().unwrap().is_empty());
        }
        InterruptedAdmission::Durable => {
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
        }
    }
}

fn directory_inventory(root: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    collect_paths(root, root, &mut paths);
    paths.sort();
    paths
}

fn collect_paths(root: &Path, directory: &Path, paths: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        paths.push(path.strip_prefix(root).unwrap().to_owned());
        if path.is_dir() {
            collect_paths(root, &path, paths);
        }
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
