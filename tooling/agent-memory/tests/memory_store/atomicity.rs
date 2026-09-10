use super::support::*;

#[test]
fn creating_a_project_scope_syncs_its_parent_before_publication()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open_with_failpoint(
        &memory_root(&root)?,
        StoreFailpoint::AfterProjectDirectoryFsync,
    )?;
    let common = fixture.path().join("common.git");
    fs::create_dir(&common)?;
    let scope_runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    let project = resolve_project(fixture.path(), &scope_runner)?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = draft_yaml(
        "project",
        "Project directory durability.",
        "project durability",
        "Established.",
        "user-decision",
        "decision:project-directory-durability",
    );

    let result = store.admit(
        &resolved(&draft, &context)?,
        Some(&project),
        &timestamp,
        &context,
    );

    assert_rejected(&result, "store_unavailable");
    let project_directory = root.join(format!("entries/project/{}", project.key().as_str()));
    assert!(project_directory.is_dir());
    assert_eq!(fs::read_dir(project_directory)?.count(), 0);
    assert!(store.list()?.entries().is_empty());
    Ok(())
}

#[test]
fn interrupted_writes_never_publish_partial_yaml_or_a_forward_index()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        assert_interrupted_state(&failpoint, InterruptedAdmission::Absent)?;
    }
    for failpoint in undurable_yaml {
        assert_interrupted_state(&failpoint, InterruptedAdmission::Renamed)?;
    }
    for failpoint in durable_yaml {
        assert_interrupted_state(&failpoint, InterruptedAdmission::Durable)?;
    }
    Ok(())
}

#[test]
fn retry_syncs_a_renamed_admission_before_reporting_duplicate()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft(
        "A retried admission becomes durable before success.",
        "retry durability",
        "Established.",
    );
    let interrupted =
        Store::open_with_failpoint(&memory_root(&root)?, StoreFailpoint::AfterYamlRename)?;
    assert_rejected(
        &interrupted.admit(&resolved(&draft, &context)?, None, &timestamp, &context),
        "store_unavailable",
    );

    let failed_retry = Store::open_with_failpoint(
        &memory_root(&root)?,
        StoreFailpoint::BeforeYamlDirectoryFsync,
    )?;
    assert_rejected(
        &failed_retry.admit(&resolved(&draft, &context)?, None, &timestamp, &context),
        "store_unavailable",
    );

    let durable_retry = Store::open(&memory_root(&root)?)?;
    assert!(matches!(
        durable_retry.admit(&resolved(&draft, &context)?, None, &timestamp, &context),
        AdmissionResult::Duplicate { .. }
    ));
    Ok(())
}

#[derive(Clone, Copy)]
enum InterruptedAdmission {
    Absent,
    Renamed,
    Durable,
}

fn assert_interrupted_state(
    failpoint: &StoreFailpoint,
    expected: InterruptedAdmission,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open_with_failpoint(&memory_root(&root)?, failpoint.clone())?;
    let before = directory_inventory(&root)?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft(
        "A durable invariant remains independently useful.",
        "durable invariant",
        "Established.",
    );

    let result = store.admit(&resolved(&draft, &context)?, None, &timestamp, &context);
    let reopened = Store::open(&memory_root(&root)?)?;
    let listing = reopened.list()?;
    let index: serde_json::Value = serde_json::from_slice(&fs::read(root.join("index.json"))?)?;

    match expected {
        InterruptedAdmission::Absent => {
            assert_rejected(&result, "store_unavailable");
            assert!(listing.entries().is_empty(), "{failpoint:?}");
            assert!(!listing.index_rebuild_required(), "{failpoint:?}");
            assert!(
                index
                    .get("entries")
                    .ok_or("missing index entries")?
                    .as_array()
                    .ok_or("missing entries array")?
                    .is_empty()
            );
            assert_eq!(directory_inventory(&root)?, before, "{failpoint:?}");
        }
        InterruptedAdmission::Renamed => {
            assert_rejected(&result, "store_unavailable");
            assert_eq!(listing.entries().len(), 1, "{failpoint:?}");
            assert!(listing.index_rebuild_required(), "{failpoint:?}");
            assert!(
                index
                    .get("entries")
                    .ok_or("missing index entries")?
                    .as_array()
                    .ok_or("missing entries array")?
                    .is_empty()
            );
        }
        InterruptedAdmission::Durable => {
            assert!(
                matches!(
                    &result,
                    AdmissionResult::Stored {
                        index_rebuild_required: true,
                        ..
                    }
                ),
                "{failpoint:?}: {result:?}"
            );
            assert_eq!(listing.entries().len(), 1, "{failpoint:?}");
            assert!(listing.index_rebuild_required(), "{failpoint:?}");
            assert!(
                index
                    .get("entries")
                    .ok_or("missing index entries")?
                    .as_array()
                    .ok_or("missing entries array")?
                    .is_empty()
            );
        }
    }
    Ok(())
}

fn directory_inventory(
    root: &Path,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut paths = Vec::new();
    collect_paths(root, root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_paths(
    root: &Path,
    directory: &Path,
    paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        paths.push(path.strip_prefix(root)?.to_owned());
        if path.is_dir() {
            collect_paths(root, &path, paths)?;
        }
    }
    Ok(())
}

#[test]
fn replace_active_atomically_updates_yaml_and_index_once()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft(
        "Transition invariant.",
        "transition invariant",
        "Established.",
    );
    let id = stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;
    let path = root.join(format!("entries/user/{id}.yaml"));
    let terminal_yaml = fs::read_to_string(&path)?.replacen(
        "status: active",
        "status: invalidated",
        1,
    ) + "transition:\n  from: active\n  to: invalidated\n  at: 2026-08-28T13:00:00Z\n  verdict: invalid\n  reason: The proof changed.\n";
    let terminal = parse_entry(terminal_yaml.as_bytes())?;

    let commit = store.replace_active(&terminal)?;

    assert!(!commit.index_rebuild_required());
    assert_eq!(
        store.load(&id)?.ok_or("missing fixture value")?.status(),
        Status::Invalidated
    );
    let listing = store.list()?;
    assert!(!listing.index_rebuild_required());
    let index: serde_json::Value = serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
    assert!(
        index
            .get("entries")
            .ok_or("missing index entries")?
            .as_array()
            .ok_or("missing fixture value")?
            .is_empty()
    );
    assert_eq!(
        store
            .replace_active(&terminal)
            .err()
            .ok_or("expected operation failure")?
            .code(),
        "entry_not_active"
    );
    Ok(())
}

#[test]
fn replace_active_refuses_changes_to_immutable_entry_fields()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft("Immutable invariant.", "original term", "Established.");
    let id = stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;
    let path = root.join(format!("entries/user/{id}.yaml"));
    let before = fs::read(&path)?;
    let terminal_yaml = String::from_utf8(before.clone())?
        .replacen("status: active", "status: invalidated", 1)
        .replacen("- original term", "- changed term", 1)
        + "transition:\n  from: active\n  to: invalidated\n  at: 2026-08-28T13:00:00Z\n  verdict: invalid\n  reason: The proof changed.\n";
    let terminal = parse_entry(terminal_yaml.as_bytes())?;

    let error = store
        .replace_active(&terminal)
        .err()
        .ok_or("expected operation failure")?;

    assert_eq!(error.code(), "entry_conflict");
    assert_eq!(fs::read(path)?, before);
    Ok(())
}

#[test]
fn replace_active_refuses_changes_to_original_timestamps()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for field in ["created_at", "established_at"] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let store = Store::open(&memory_root(&root)?)?;
        let git = FakeProcessRunner::default();
        let curl = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &git, &curl);
        let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
        let draft = user_draft("Timestamp invariant.", "timestamp", "Established.");
        let id = stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;
        let path = root.join(format!("entries/user/{id}.yaml"));
        let before = fs::read(&path)?;
        let terminal_yaml = String::from_utf8(before.clone())?
            .replacen("status: active", "status: invalidated", 1)
            .replacen(
                &format!("{field}: 2026-08-28T12:00:00Z"),
                &format!("{field}: 2026-08-28T12:01:00Z"),
                1,
            )
            + "transition:\n  from: active\n  to: invalidated\n  at: 2026-08-28T13:00:00Z\n  verdict: invalid\n  reason: The proof changed.\n";
        let terminal = parse_entry(terminal_yaml.as_bytes())?;

        let error = store
            .replace_active(&terminal)
            .err()
            .ok_or("expected operation failure")?;

        assert_eq!(error.code(), "entry_conflict", "{field}");
        assert_eq!(fs::read(path)?, before, "{field}");
    }
    Ok(())
}
