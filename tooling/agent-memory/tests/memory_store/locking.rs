use super::support::*;

#[test]
fn a_replaced_lock_never_opens_a_second_critical_section() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let barrier = Arc::new(Barrier::new(2));
    let first_store = Store::open_with_failpoint(
        memory_root(&root),
        StoreFailpoint::PauseAfterLockAcquire(Arc::clone(&barrier)),
    )
    .unwrap();
    let second_store = Store::open(memory_root(&root)).unwrap();
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let draft = user_draft("Lock inode identity.", "lock inode", "Established.");
    let first_resolved = resolved(&draft, &context);
    let second_resolved = resolved(&draft, &context);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let first_timestamp = timestamp.clone();
    let cwd = fixture.path().to_owned();

    let first = std::thread::spawn(move || {
        let runner = SystemProcessRunner;
        let context = SourceContext::new(&cwd, &runner, &runner);
        first_store.admit(first_resolved, None, &first_timestamp, &context)
    });
    barrier.wait();
    fs::rename(root.join(".lock"), root.join(".lock-displaced")).unwrap();
    fs::write(root.join(".lock"), b"").unwrap();
    let second_result = second_store.admit(second_resolved, None, &timestamp, &context);
    barrier.wait();
    let first_result = first.join().unwrap();

    assert_conflict(second_result, "store_lock_timeout");
    assert_conflict(first_result, "store_lock_unavailable");
    assert!(second_store.list().unwrap().entries().is_empty());
}

#[test]
fn refuses_an_existing_or_hardlinked_final_entry_without_overwriting_it() {
    for hardlinked in [false, true] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        let id = "mem_2253f44de1abe942c38b0d64";
        let final_path = root.join(format!("entries/user/{id}.yaml"));
        let outside = fixture.path().join("outside");
        if hardlinked {
            fs::write(&outside, b"external bytes").unwrap();
            fs::hard_link(&outside, &final_path).unwrap();
        } else {
            fs::write(&final_path, b"occupied bytes").unwrap();
        }
        let before = fs::read(&final_path).unwrap();
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

        assert_conflict(result, "entry_conflict");
        assert_eq!(fs::read(&final_path).unwrap(), before);
        if hardlinked {
            assert_eq!(fs::read(outside).unwrap(), before);
        }
    }
}

#[test]
fn refuses_a_missing_symlinked_or_timed_out_global_lock() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft("Lock invariant.", "lock invariant", "Established.");
    fs::remove_file(root.join(".lock")).unwrap();

    assert_conflict(
        store.admit(resolved(&draft, &context), None, &timestamp, &context),
        "store_lock_unavailable",
    );

    let store = Store::open(memory_root(&root)).unwrap();
    fs::remove_file(root.join(".lock")).unwrap();
    let outside = fixture.path().join("outside-lock");
    fs::write(&outside, b"unchanged").unwrap();
    symlink(&outside, root.join(".lock")).unwrap();
    assert_conflict(
        store.admit(resolved(&draft, &context), None, &timestamp, &context),
        "store_lock_unavailable",
    );
    assert_eq!(fs::read(outside).unwrap(), b"unchanged");

    fs::remove_file(root.join(".lock")).unwrap();
    let store = Store::open(memory_root(&root)).unwrap();
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".lock"))
        .unwrap();
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive).unwrap();
    let started = Instant::now();
    let result = store.admit(resolved(&draft, &context), None, &timestamp, &context);
    let elapsed = started.elapsed();

    assert_conflict(result, "store_lock_timeout");
    assert!(elapsed >= Duration::from_secs(2), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(3), "{elapsed:?}");
}

#[test]
fn acquires_the_lock_before_creating_a_project_scope_directory() {
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
    let draft = draft_yaml(
        "project",
        "Project lock invariant.",
        "project lock",
        "Established.",
        "user-decision",
        "decision:project-lock",
    );
    fs::remove_file(root.join(".lock")).unwrap();

    let result = store.admit(
        resolved(&draft, &context),
        Some(&project),
        &timestamp,
        &context,
    );

    assert_conflict(result, "store_lock_unavailable");
    assert!(
        !root
            .join(format!("entries/project/{}", project.key().as_str()))
            .exists()
    );
}

#[test]
fn repairs_the_lock_mode_again_when_acquiring_it() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let lock = root.join(".lock");
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o666)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft("Lock mode invariant.", "lock mode", "Established.");

    stored_id(store.admit(resolved(&draft, &context), None, &timestamp, &context));

    assert_eq!(private_mode(&lock), 0o600);
}

#[test]
fn refuses_controlled_symlinks_installed_after_open_and_keeps_the_root_descriptor_anchored() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let outside = fixture.path().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("sentinel"), b"unchanged").unwrap();
    fs::remove_dir(root.join("entries/user")).unwrap();
    symlink(&outside, root.join("entries/user")).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = user_draft("Symlink invariant.", "symlink", "Established.");

    assert_conflict(
        store.admit(resolved(&draft, &context), None, &timestamp, &context),
        "entry_conflict",
    );
    assert_eq!(fs::read(outside.join("sentinel")).unwrap(), b"unchanged");
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);

    fs::remove_file(root.join("entries/user")).unwrap();
    fs::create_dir(root.join("entries/user")).unwrap();
    let anchored = fixture.path().join("anchored");
    fs::rename(&root, &anchored).unwrap();
    fs::create_dir(&root).unwrap();
    symlink(&outside, root.join("entries")).unwrap();
    let id = stored_id(store.admit(resolved(&draft, &context), None, &timestamp, &context));

    assert!(anchored.join(format!("entries/user/{id}.yaml")).is_file());
    assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
}
