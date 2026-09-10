use super::support::*;

#[test]
fn conflicts_when_a_source_changes_after_resolution_under_the_lock()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let source = fixture.path().join("proof.txt");
    fs::write(&source, b"before")?;
    let root = fixture.path().join("store");
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        &MemoryRoot::new(&root)?,
        StoreFailpoint::PauseAfterLockAcquire(Arc::clone(&barrier)),
    )?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;
    let bytes = draft(
        Some("user"),
        "invariant",
        "Source remains stable.",
        "local-file",
        source.to_str().ok_or("missing fixture value")?,
    );

    let result = std::thread::scope(
        |scope| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
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
            });
            barrier.wait();
            let written = fs::write(&source, b"after");
            barrier.wait();
            written?;
            Ok(worker.join().map_err(|_| "worker thread panicked")??)
        },
    )?;

    assert_conflict(&result, "source_changed");
    assert!(store.list()?.entries().is_empty());
    Ok(())
}

#[test]
fn rechecks_the_source_after_staging_at_the_yaml_publication_boundary()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let source = fixture.path().join("proof.txt");
    fs::write(&source, b"before")?;
    let root = fixture.path().join("store");
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        &MemoryRoot::new(&root)?,
        StoreFailpoint::PauseBeforeYamlRename(Arc::clone(&barrier)),
    )?;
    let before = directory_inventory(&root)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;
    let bytes = draft(
        Some("user"),
        "invariant",
        "Publication observes the final source snapshot.",
        "local-file",
        source.to_str().ok_or("missing fixture value")?,
    );

    let result = std::thread::scope(
        |scope| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
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
            });
            barrier.wait();
            let written = fs::write(&source, b"after staging");
            barrier.wait();
            written?;
            Ok(worker.join().map_err(|_| "worker thread panicked")??)
        },
    )?;

    assert_conflict(&result, "source_changed");
    assert!(store.list()?.entries().is_empty());
    assert_eq!(directory_inventory(&root)?, before);
    Ok(())
}

fn directory_inventory(
    root: &std::path::Path,
) -> Result<Vec<std::path::PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut paths = Vec::new();
    collect_paths(root, root, &mut paths)?;
    paths.sort();
    Ok(paths)
}

fn collect_paths(
    root: &std::path::Path,
    path: &std::path::Path,
    paths: &mut Vec<std::path::PathBuf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        paths.push(path.strip_prefix(root)?.to_owned());
        if path.is_dir() {
            collect_paths(root, &path, paths)?;
        }
    }
    Ok(())
}

#[test]
fn retry_after_an_undurable_yaml_result_returns_duplicate()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("store");
    let failing_store =
        Store::open_with_failpoint(&MemoryRoot::new(&root)?, StoreFailpoint::AfterYamlRename)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;
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
    )?;
    assert_rejected(&first, "store_unavailable");
    let store = Store::open(&MemoryRoot::new(&root)?)?;
    let retry = admit(
        &bytes,
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ExplicitRequest,
        ),
    )?;

    let entries = store.list()?;
    assert!(
        matches!(&retry, AdmissionResult::Duplicate { id } if entries.entries().first().is_some_and(|entry| entry.id() == id)),
        "unexpected result: {retry:?}"
    );
    assert_eq!(store.list()?.entries().len(), 1);
    Ok(())
}
