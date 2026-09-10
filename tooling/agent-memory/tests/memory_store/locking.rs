use super::support::*;

#[test]
fn a_replaced_lock_never_opens_a_second_critical_section()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let barrier = Arc::new(Barrier::new(2));
    let first_store = Store::open_with_failpoint(
        &memory_root(&root)?,
        StoreFailpoint::PauseAfterLockAcquire(Arc::clone(&barrier)),
    )?;
    let second_store = Store::open(&memory_root(&root)?)?;
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let draft = user_draft("Lock inode identity.", "lock inode", "Established.");
    let first_resolved = resolved(&draft, &context)?;
    let second_resolved = resolved(&draft, &context)?;
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let first_timestamp = timestamp.clone();
    let cwd = fixture.path().to_owned();

    let first = std::thread::spawn(move || {
        let runner = SystemProcessRunner;
        let context = SourceContext::new(&cwd, &runner, &runner);
        first_store.admit(&first_resolved, None, &first_timestamp, &context)
    });
    barrier.wait();
    fs::rename(root.join(".lock"), root.join(".lock-displaced"))?;
    fs::write(root.join(".lock"), b"")?;
    let second_result = second_store.admit(&second_resolved, None, &timestamp, &context);
    barrier.wait();
    let first_result = first.join().map_err(|_| "worker thread panicked")?;

    assert_conflict(&second_result, "store_lock_timeout");
    assert_conflict(&first_result, "store_lock_unavailable");
    assert!(second_store.list()?.entries().is_empty());
    Ok(())
}

#[test]
fn refuses_an_existing_or_hardlinked_final_entry_without_overwriting_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for hardlinked in [false, true] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let store = Store::open(&memory_root(&root)?)?;
        let id = "mem_2253f44de1abe942c38b0d64";
        let final_path = root.join(format!("entries/user/{id}.yaml"));
        let outside = fixture.path().join("outside");
        if hardlinked {
            fs::write(&outside, b"external bytes")?;
            fs::hard_link(&outside, &final_path)?;
        } else {
            fs::write(&final_path, b"occupied bytes")?;
        }
        let before = fs::read(&final_path)?;
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

        assert_conflict(&result, "entry_conflict");
        assert_eq!(fs::read(&final_path)?, before);
        if hardlinked {
            assert_eq!(fs::read(outside)?, before);
        }
    }
    Ok(())
}

#[test]
fn refuses_a_missing_symlinked_or_timed_out_global_lock()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft("Lock invariant.", "lock invariant", "Established.");
    fs::remove_file(root.join(".lock"))?;

    assert_conflict(
        &store.admit(&resolved(&draft, &context)?, None, &timestamp, &context),
        "store_lock_unavailable",
    );

    let store = Store::open(&memory_root(&root)?)?;
    fs::remove_file(root.join(".lock"))?;
    let outside = fixture.path().join("outside-lock");
    fs::write(&outside, b"unchanged")?;
    symlink(&outside, root.join(".lock"))?;
    assert_conflict(
        &store.admit(&resolved(&draft, &context)?, None, &timestamp, &context),
        "store_lock_unavailable",
    );
    assert_eq!(fs::read(outside)?, b"unchanged");

    fs::remove_file(root.join(".lock"))?;
    let store = Store::open(&memory_root(&root)?)?;
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(root.join(".lock"))?;
    rustix::fs::flock(&lock, rustix::fs::FlockOperation::LockExclusive)?;
    let started = Instant::now();
    let result = store.admit(&resolved(&draft, &context)?, None, &timestamp, &context);
    let elapsed = started.elapsed();

    assert_conflict(&result, "store_lock_timeout");
    assert!(elapsed >= Duration::from_secs(2), "{elapsed:?}");
    assert!(elapsed < Duration::from_secs(3), "{elapsed:?}");
    Ok(())
}

#[test]
fn acquires_the_lock_before_creating_a_project_scope_directory()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
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
        "Project lock invariant.",
        "project lock",
        "Established.",
        "user-decision",
        "decision:project-lock",
    );
    fs::remove_file(root.join(".lock"))?;

    let result = store.admit(
        &resolved(&draft, &context)?,
        Some(&project),
        &timestamp,
        &context,
    );

    assert_conflict(&result, "store_lock_unavailable");
    assert!(
        !root
            .join(format!("entries/project/{}", project.key().as_str()))
            .exists()
    );
    Ok(())
}

#[test]
fn repairs_the_lock_mode_again_when_acquiring_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let lock = root.join(".lock");
    fs::set_permissions(&lock, fs::Permissions::from_mode(0o666))?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft("Lock mode invariant.", "lock mode", "Established.");

    stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;

    assert_eq!(private_mode(&lock)?, 0o600);
    Ok(())
}

#[test]
fn refuses_controlled_symlinks_installed_after_open_and_keeps_the_root_descriptor_anchored()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let outside = fixture.path().join("outside");
    fs::create_dir(&outside)?;
    fs::write(outside.join("sentinel"), b"unchanged")?;
    fs::remove_dir(root.join("entries/user"))?;
    symlink(&outside, root.join("entries/user"))?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft("Symlink invariant.", "symlink", "Established.");

    assert_conflict(
        &store.admit(&resolved(&draft, &context)?, None, &timestamp, &context),
        "entry_conflict",
    );
    assert_eq!(fs::read(outside.join("sentinel"))?, b"unchanged");
    assert_eq!(fs::read_dir(&outside)?.count(), 1);

    fs::remove_file(root.join("entries/user"))?;
    fs::create_dir(root.join("entries/user"))?;
    let anchored = fixture.path().join("anchored");
    fs::rename(&root, &anchored)?;
    fs::create_dir(&root)?;
    symlink(&outside, root.join("entries"))?;
    let id = stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;

    assert!(anchored.join(format!("entries/user/{id}.yaml")).is_file());
    assert_eq!(fs::read_dir(&outside)?.count(), 1);
    Ok(())
}
