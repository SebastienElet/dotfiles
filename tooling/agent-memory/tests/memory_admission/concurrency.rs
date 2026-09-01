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
fn rechecks_the_source_after_staging_at_the_yaml_publication_boundary() {
    let fixture = tempfile::tempdir().unwrap();
    let source = fixture.path().join("proof.txt");
    fs::write(&source, b"before").unwrap();
    let root = fixture.path().join("store");
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        MemoryRoot::new(&root).unwrap(),
        StoreFailpoint::PauseBeforeYamlRename(Arc::clone(&barrier)),
    )
    .unwrap();
    let before = directory_inventory(&root);
    let processes = SystemProcessRunner;
    let clock = FixedClock::new();
    let bytes = draft(
        Some("user"),
        "invariant",
        "Publication observes the final source snapshot.",
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
        fs::write(&source, b"after staging").unwrap();
        barrier.wait();
        worker.join().unwrap()
    });

    assert_conflict(result, "source_changed");
    assert!(store.list().unwrap().entries().is_empty());
    assert_eq!(directory_inventory(&root), before);
}

fn directory_inventory(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    collect_paths(root, root, &mut paths);
    paths.sort();
    paths
}

fn collect_paths(
    root: &std::path::Path,
    path: &std::path::Path,
    paths: &mut Vec<std::path::PathBuf>,
) {
    for entry in fs::read_dir(path).unwrap() {
        let path = entry.unwrap().path();
        paths.push(path.strip_prefix(root).unwrap().to_owned());
        if path.is_dir() {
            collect_paths(root, &path, paths);
        }
    }
}

#[test]
fn retry_after_an_undurable_yaml_result_returns_duplicate() {
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
    assert_rejected(first, "store_unavailable");
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
        AdmissionResult::Duplicate { id } => {
            assert_eq!(store.list().unwrap().entries()[0].id(), &id)
        }
        result => panic!("unexpected result: {result:?}"),
    }
    assert_eq!(store.list().unwrap().entries().len(), 1);
}
