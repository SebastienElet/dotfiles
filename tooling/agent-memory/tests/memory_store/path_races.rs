use super::support::*;
use std::path::PathBuf;

#[derive(Clone, Copy, Debug)]
enum Substitution {
    File,
    Hardlink,
    Symlink,
}

#[test]
fn refuses_a_substituted_yaml_temporary_before_publication() {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            memory_root(&root),
            StoreFailpoint::PauseBeforeYamlRename(Arc::clone(&barrier)),
        )
        .unwrap();
        let runner = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &runner, &runner);
        let draft = user_draft("Temporary identity.", "temporary identity", "Established.");
        let resolved = resolved(&draft, &context);
        let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
        let cwd = fixture.path().to_owned();

        let admission = std::thread::spawn(move || {
            let runner = SystemProcessRunner;
            let context = SourceContext::new(&cwd, &runner, &runner);
            store.admit(resolved, None, &timestamp, &context)
        });
        barrier.wait();
        let temporary = only_temporary(&root.join("entries/user"));
        let outside = substitute(&temporary, substitution);
        barrier.wait();
        let result = admission.join().unwrap();

        assert_rejected(result, "unsafe_store_path");
        assert_eq!(fs::read(outside).unwrap(), b"substituted");
        assert_eq!(yaml_paths(&root.join("entries/user")).len(), 0);
    }
}

#[test]
fn refuses_a_substituted_index_temporary_without_publishing_it() {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            memory_root(&root),
            StoreFailpoint::PauseBeforeIndexRename(Arc::clone(&barrier)),
        )
        .unwrap();
        let runner = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &runner, &runner);
        let draft = user_draft("Index identity.", "index identity", "Established.");
        let resolved = resolved(&draft, &context);
        let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
        let cwd = fixture.path().to_owned();

        let admission = std::thread::spawn(move || {
            let runner = SystemProcessRunner;
            let context = SourceContext::new(&cwd, &runner, &runner);
            store.admit(resolved, None, &timestamp, &context)
        });
        barrier.wait();
        let temporary = only_temporary(&root);
        let outside = substitute(&temporary, substitution);
        barrier.wait();
        let result = admission.join().unwrap();

        match result {
            AdmissionResult::Stored {
                index_rebuild_required: true,
                ..
            } => {}
            result => panic!("{substitution:?}: {result:?}"),
        }
        assert_eq!(fs::read(outside).unwrap(), b"substituted");
        let index: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
        assert!(index["entries"].as_array().unwrap().is_empty());
        assert_eq!(yaml_paths(&root.join("entries/user")).len(), 1);
    }
}

fn only_temporary(directory: &Path) -> PathBuf {
    let paths = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .contains(".tmp-")
        })
        .collect::<Vec<_>>();
    assert_eq!(paths.len(), 1, "{}", directory.display());
    paths.into_iter().next().unwrap()
}

fn substitute(temporary: &Path, substitution: Substitution) -> PathBuf {
    let displaced = temporary.with_extension("displaced");
    fs::rename(temporary, &displaced).unwrap();
    let outside = temporary.with_extension("outside");
    fs::write(&outside, b"substituted").unwrap();
    match substitution {
        Substitution::File => fs::copy(&outside, temporary).map(|_| ()).unwrap(),
        Substitution::Hardlink => fs::hard_link(&outside, temporary).unwrap(),
        Substitution::Symlink => symlink(&outside, temporary).unwrap(),
    }
    outside
}

fn yaml_paths(directory: &Path) -> Vec<PathBuf> {
    fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.extension()
                .is_some_and(|extension| extension == "yaml")
        })
        .collect()
}
