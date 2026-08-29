use super::support::*;

#[test]
fn conflicts_when_a_source_changes_after_resolution_under_the_lock() {
    let fixture = tempfile::tempdir().unwrap();
    let source = fixture.path().join("proof.txt");
    fs::write(&source, b"before").unwrap();
    let root = fixture.path().join("store");
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        MemoryRoot::new(&root).unwrap(),
        StoreFailpoint::PauseAfterLockAcquire(Arc::clone(&barrier)),
    )
    .unwrap();
    let processes = SystemProcessRunner;
    let clock = FixedClock::new();
    let bytes = draft(
        Some("user"),
        "invariant",
        "Source remains stable.",
        "local-file",
        source.to_str().unwrap(),
    );

    let result = std::thread::scope(|scope| {
        let worker = scope.spawn(|| {
            admit(
                &bytes,
                context(
                    &store,
                    fixture.path(),
                    &clock,
                    &processes,
                    AdmissionAuthorization::ExplicitRequest,
                ),
            )
            .unwrap()
        });
        barrier.wait();
        fs::write(&source, b"after").unwrap();
        barrier.wait();
        worker.join().unwrap()
    });

    assert_conflict(result, "source_changed");
    assert!(store.list().unwrap().entries().is_empty());
}

#[test]
fn retry_after_a_post_yaml_failure_returns_duplicate() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("store");
    let failing_store = Store::open_with_failpoint(
        MemoryRoot::new(&root).unwrap(),
        StoreFailpoint::AfterYamlRename,
    )
    .unwrap();
    let processes = SystemProcessRunner;
    let clock = FixedClock::new();
    let bytes = draft(
        Some("user"),
        "invariant",
        "Retryable memory.",
        "user-decision",
        "decision:retryable-memory",
    );

    let first = admit(
        &bytes,
        context(
            &failing_store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ExplicitRequest,
        ),
    )
    .unwrap();
    let id = stored_id(first, true);
    let store = Store::open(MemoryRoot::new(&root).unwrap()).unwrap();
    let retry = admit(
        &bytes,
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ExplicitRequest,
        ),
    )
    .unwrap();

    match retry {
        AdmissionResult::Duplicate { id: duplicate } => assert_eq!(duplicate.as_str(), id),
        result => panic!("unexpected result: {result:?}"),
    }
    assert_eq!(store.list().unwrap().entries().len(), 1);
}
