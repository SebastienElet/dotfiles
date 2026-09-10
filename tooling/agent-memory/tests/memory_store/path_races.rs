use super::support::*;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
enum Substitution {
    File,
    Hardlink,
    Symlink,
}

#[test]
fn refuses_a_substituted_yaml_temporary_before_publication()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            &memory_root(&root)?,
            StoreFailpoint::PauseBeforeYamlRename(Arc::clone(&barrier)),
        )?;
        let runner = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &runner, &runner);
        let draft = user_draft("Temporary identity.", "temporary identity", "Established.");
        let resolved = resolved(&draft, &context)?;
        let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
        let cwd = fixture.path().to_owned();

        let admission = std::thread::spawn(move || {
            let runner = SystemProcessRunner;
            let context = SourceContext::new(&cwd, &runner, &runner);
            store.admit(&resolved, None, &timestamp, &context)
        });
        barrier.wait();
        let temporary = only_temporary(&root.join("entries/user"))?;
        let outside = substitute(&temporary, substitution)?;
        barrier.wait();
        let result = admission.join().map_err(|_| "worker thread panicked")?;

        assert_rejected(&result, "unsafe_store_path");
        assert_eq!(fs::read(outside)?, b"substituted");
        assert_eq!(yaml_paths(&root.join("entries/user"))?.len(), 0);
        assert!(temporary_paths(&root)?.is_empty());
    }
    Ok(())
}

#[test]
fn refuses_a_substituted_index_temporary_without_publishing_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            &memory_root(&root)?,
            StoreFailpoint::PauseBeforeIndexRename(Arc::clone(&barrier)),
        )?;
        let runner = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &runner, &runner);
        let draft = user_draft("Index identity.", "index identity", "Established.");
        let resolved = resolved(&draft, &context)?;
        let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
        let cwd = fixture.path().to_owned();

        let admission = std::thread::spawn(move || {
            let runner = SystemProcessRunner;
            let context = SourceContext::new(&cwd, &runner, &runner);
            store.admit(&resolved, None, &timestamp, &context)
        });
        barrier.wait();
        let temporary = only_temporary(&root)?;
        let outside = substitute(&temporary, substitution)?;
        barrier.wait();
        let result = admission.join().map_err(|_| "worker thread panicked")?;

        assert!(
            matches!(
                &result,
                AdmissionResult::Stored {
                    index_rebuild_required: true,
                    ..
                }
            ),
            "{substitution:?}: {result:?}"
        );
        assert_eq!(fs::read(outside)?, b"substituted");
        let index: serde_json::Value = serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
        assert!(
            index
                .get("entries")
                .ok_or("missing index entries")?
                .as_array()
                .ok_or("missing fixture value")?
                .is_empty()
        );
        assert_eq!(yaml_paths(&root.join("entries/user"))?.len(), 1);
    }
    Ok(())
}

fn only_temporary(directory: &Path) -> std::io::Result<PathBuf> {
    let mut paths = temporary_paths(directory)?;
    assert_eq!(paths.len(), 1, "{}", directory.display());
    paths
        .pop()
        .ok_or_else(|| std::io::Error::other("missing temporary file"))
}

fn temporary_paths(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.file_name()
            .is_some_and(|name| name.to_string_lossy().contains(".tmp-"))
    });
    Ok(paths)
}

fn substitute(temporary: &Path, substitution: Substitution) -> std::io::Result<PathBuf> {
    let displaced = temporary.with_extension("displaced");
    fs::rename(temporary, &displaced)?;
    let outside = temporary.with_extension("outside");
    fs::write(&outside, b"substituted")?;
    match substitution {
        Substitution::File => {
            fs::copy(&outside, temporary)?;
        }
        Substitution::Hardlink => fs::hard_link(&outside, temporary)?,
        Substitution::Symlink => symlink(&outside, temporary)?,
    }
    Ok(outside)
}

fn yaml_paths(directory: &Path) -> std::io::Result<Vec<PathBuf>> {
    let mut paths = fs::read_dir(directory)?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.retain(|path| {
        path.extension()
            .is_some_and(|extension| extension == "yaml")
    });
    Ok(paths)
}
