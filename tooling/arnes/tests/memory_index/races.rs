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

#[derive(Clone, Copy, Debug)]
enum InventoryMutation {
    Add,
    Delete,
    Substitute,
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

#[test]
fn yaml_inventory_changes_immediately_before_publication_fail_closed() {
    for mutation in [
        InventoryMutation::Add,
        InventoryMutation::Delete,
        InventoryMutation::Substitute,
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let initial = Store::open(memory_root(&root)).unwrap();
        let id = admit_user(
            &initial,
            fixture.path(),
            "Final inventory identity.",
            &["final inventory"],
            "Established.",
        );
        let yaml = root.join(format!("entries/user/{id}.yaml"));
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
        mutate_inventory(&yaml, mutation);
        barrier.wait();

        let error = rebuild.join().unwrap().unwrap_err();

        assert_eq!(error.code(), "unsafe_store_path", "{mutation:?}");
        assert_eq!(fs::read(index).unwrap(), b"corrupt", "{mutation:?}");
    }
}

#[test]
fn yaml_substitution_after_anchored_read_never_combines_content_and_metadata() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let initial = Store::open(memory_root(&root)).unwrap();
    let id = admit_user(
        &initial,
        fixture.path(),
        "Original anchored content.",
        &["original anchored"],
        "Established.",
    );
    let yaml = root.join(format!("entries/user/{id}.yaml"));
    fs::write(root.join("index.json"), b"corrupt").unwrap();
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        memory_root(&root),
        StoreFailpoint::PauseAfterIndexEntryRead(Arc::clone(&barrier)),
    )
    .unwrap();
    let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
    barrier.wait();
    let original = fs::read_to_string(&yaml).unwrap();
    fs::rename(&yaml, yaml.with_extension("displaced")).unwrap();
    fs::write(
        &yaml,
        original.replace("Original anchored content.", "Substituted content now."),
    )
    .unwrap();
    barrier.wait();

    let error = rebuild.join().unwrap().unwrap_err();

    assert_eq!(error.code(), "unsafe_store_path");
    assert_eq!(fs::read(root.join("index.json")).unwrap(), b"corrupt");
}

fn mutate_inventory(yaml: &Path, mutation: InventoryMutation) {
    match mutation {
        InventoryMutation::Add => {
            fs::copy(
                yaml,
                yaml.with_file_name("mem_ffffffffffffffffffffffff.yaml"),
            )
            .unwrap();
        }
        InventoryMutation::Delete => fs::remove_file(yaml).unwrap(),
        InventoryMutation::Substitute => {
            let displaced = yaml.with_extension("displaced");
            fs::rename(yaml, &displaced).unwrap();
            fs::copy(displaced, yaml).unwrap();
        }
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
