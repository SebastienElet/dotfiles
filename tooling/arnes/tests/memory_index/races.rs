use super::support::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};

#[derive(Clone, Copy, Debug)]
enum Substitution {
    File,
    Hardlink,
    Symlink,
}

#[test]
fn yaml_substitution_between_inventory_and_parse_fails_closed() {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let initial = Store::open(memory_root(&root)).unwrap();
        let id = admit_user(
            &initial,
            fixture.path(),
            "Anchored rebuild statement.",
            &["anchored rebuild"],
            "Established.",
        );
        let yaml = root.join(format!("entries/user/{id}.yaml"));
        let authority_before = fs::read(&yaml).unwrap();
        fs::write(root.join("index.json"), b"corrupt").unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            memory_root(&root),
            StoreFailpoint::PauseBeforeIndexEntryRead(Arc::clone(&barrier)),
        )
        .unwrap();
        let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
        barrier.wait();
        let outside = substitute(&yaml, substitution);
        barrier.wait();

        let error = rebuild.join().unwrap().unwrap_err();

        assert_eq!(error.code(), "unsafe_store_path", "{substitution:?}");
        assert_eq!(fs::read(&outside).unwrap(), b"substituted");
        assert_eq!(fs::read(root.join("index.json")).unwrap(), b"corrupt");
        assert_ne!(fs::read(yaml).unwrap(), authority_before);
    }
}

#[test]
fn final_index_substitution_after_staging_fails_closed() {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let initial = Store::open(memory_root(&root)).unwrap();
        admit_user(
            &initial,
            fixture.path(),
            "Final index identity.",
            &["final index"],
            "Established.",
        );
        let index = root.join("index.json");
        fs::write(&index, b"corrupt").unwrap();
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            memory_root(&root),
            StoreFailpoint::PauseBeforeIndexRename(Arc::clone(&barrier)),
        )
        .unwrap();
        let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
        barrier.wait();
        let outside = substitute(&index, substitution);
        barrier.wait();

        let error = rebuild.join().unwrap().unwrap_err();

        assert_eq!(error.code(), "unsafe_store_path", "{substitution:?}");
        assert_eq!(fs::read(&outside).unwrap(), b"substituted");
        assert_eq!(fs::read(index).unwrap(), b"substituted");
    }
}

fn substitute(yaml: &Path, substitution: Substitution) -> PathBuf {
    let displaced = yaml.with_extension("displaced");
    fs::rename(yaml, displaced).unwrap();
    let outside = yaml.with_extension("outside");
    fs::write(&outside, b"substituted").unwrap();
    match substitution {
        Substitution::File => fs::copy(&outside, yaml).map(|_| ()).unwrap(),
        Substitution::Hardlink => fs::hard_link(&outside, yaml).unwrap(),
        Substitution::Symlink => symlink(&outside, yaml).unwrap(),
    }
    outside
}
