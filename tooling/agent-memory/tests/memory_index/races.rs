use super::support::*;
use agent_memory::{Index, StoreFailpoint};
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
fn yaml_substitution_between_inventory_and_parse_fails_closed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let initial = Store::open(&memory_root(&root)?)?;
        let id = admit_user(
            &initial,
            fixture.path(),
            "Anchored rebuild statement.",
            &["anchored rebuild"],
            "Established.",
        )?;
        let yaml = root.join(format!("entries/user/{id}.yaml"));
        let authority_before = fs::read(&yaml)?;
        fs::write(root.join("index.json"), b"corrupt")?;
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            &memory_root(&root)?,
            StoreFailpoint::PauseBeforeIndexEntryRead(Arc::clone(&barrier)),
        )?;
        let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
        barrier.wait();
        let outside = substitute(&yaml, substitution)?;
        barrier.wait();

        let error = rebuild
            .join()
            .map_err(|_| "worker thread panicked")?
            .err()
            .ok_or("expected operation failure")?;

        assert_eq!(error.code(), "unsafe_store_path", "{substitution:?}");
        assert_eq!(fs::read(&outside)?, b"substituted");
        assert_eq!(fs::read(root.join("index.json"))?, b"corrupt");
        assert_ne!(fs::read(yaml)?, authority_before);
    }
    Ok(())
}

#[test]
fn final_index_substitution_after_staging_fails_closed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for substitution in [
        Substitution::File,
        Substitution::Hardlink,
        Substitution::Symlink,
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let initial = Store::open(&memory_root(&root)?)?;
        admit_user(
            &initial,
            fixture.path(),
            "Final index identity.",
            &["final index"],
            "Established.",
        )?;
        let index = root.join("index.json");
        fs::write(&index, b"corrupt")?;
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            &memory_root(&root)?,
            StoreFailpoint::PauseBeforeIndexRename(Arc::clone(&barrier)),
        )?;
        let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
        barrier.wait();
        let outside = substitute(&index, substitution)?;
        barrier.wait();

        let error = rebuild
            .join()
            .map_err(|_| "worker thread panicked")?
            .err()
            .ok_or("expected operation failure")?;

        assert_eq!(error.code(), "unsafe_store_path", "{substitution:?}");
        assert_eq!(fs::read(&outside)?, b"substituted");
        assert_eq!(fs::read(index)?, b"substituted");
    }
    Ok(())
}

#[test]
fn yaml_inventory_changes_immediately_before_publication_fail_closed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for mutation in [
        InventoryMutation::Add,
        InventoryMutation::Delete,
        InventoryMutation::Substitute,
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let initial = Store::open(&memory_root(&root)?)?;
        let id = admit_user(
            &initial,
            fixture.path(),
            "Final inventory identity.",
            &["final inventory"],
            "Established.",
        )?;
        let yaml = root.join(format!("entries/user/{id}.yaml"));
        let index = root.join("index.json");
        fs::write(&index, b"corrupt")?;
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            &memory_root(&root)?,
            StoreFailpoint::PauseBeforeIndexRename(Arc::clone(&barrier)),
        )?;
        let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
        barrier.wait();
        mutate_inventory(&yaml, mutation)?;
        barrier.wait();

        let error = rebuild
            .join()
            .map_err(|_| "worker thread panicked")?
            .err()
            .ok_or("expected operation failure")?;

        assert_eq!(error.code(), "unsafe_store_path", "{mutation:?}");
        assert_eq!(fs::read(index)?, b"corrupt", "{mutation:?}");
    }
    Ok(())
}

#[test]
fn yaml_substitution_after_anchored_read_never_combines_content_and_metadata()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let initial = Store::open(&memory_root(&root)?)?;
    let id = admit_user(
        &initial,
        fixture.path(),
        "Original anchored content.",
        &["original anchored"],
        "Established.",
    )?;
    let yaml = root.join(format!("entries/user/{id}.yaml"));
    fs::write(root.join("index.json"), b"corrupt")?;
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        &memory_root(&root)?,
        StoreFailpoint::PauseAfterIndexEntryRead(Arc::clone(&barrier)),
    )?;
    let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
    barrier.wait();
    let original = fs::read_to_string(&yaml)?;
    fs::rename(&yaml, yaml.with_extension("displaced"))?;
    fs::write(
        &yaml,
        original.replace("Original anchored content.", "Substituted content now."),
    )?;
    barrier.wait();

    let error = rebuild
        .join()
        .map_err(|_| "worker thread panicked")?
        .err()
        .ok_or("expected operation failure")?;

    assert_eq!(error.code(), "unsafe_store_path");
    assert_eq!(fs::read(root.join("index.json"))?, b"corrupt");
    Ok(())
}

fn mutate_inventory(yaml: &Path, mutation: InventoryMutation) -> std::io::Result<()> {
    match mutation {
        InventoryMutation::Add => {
            fs::copy(
                yaml,
                yaml.with_file_name("mem_ffffffffffffffffffffffff.yaml"),
            )?;
        }
        InventoryMutation::Delete => fs::remove_file(yaml)?,
        InventoryMutation::Substitute => {
            let displaced = yaml.with_extension("displaced");
            fs::rename(yaml, &displaced)?;
            fs::copy(displaced, yaml)?;
        }
    }
    Ok(())
}

fn substitute(yaml: &Path, substitution: Substitution) -> std::io::Result<PathBuf> {
    let displaced = yaml.with_extension("displaced");
    fs::rename(yaml, displaced)?;
    let outside = yaml.with_extension("outside");
    fs::write(&outside, b"substituted")?;
    match substitution {
        Substitution::File => fs::copy(&outside, yaml).map(|_| ())?,
        Substitution::Hardlink => fs::hard_link(&outside, yaml)?,
        Substitution::Symlink => symlink(&outside, yaml)?,
    }
    Ok(outside)
}
