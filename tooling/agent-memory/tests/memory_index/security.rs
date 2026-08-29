use super::support::*;
use std::fs;
use std::os::unix::fs::symlink;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

#[test]
fn index_symlink_hardlink_and_yaml_link_bypasses_fail_closed() {
    for bypass in [
        "index-symlink",
        "index-hardlink",
        "yaml-symlink",
        "yaml-hardlink",
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        let id = admit_user(
            &store,
            fixture.path(),
            "Linked entry statement.",
            &["linked entry"],
            "Established.",
        );
        let index = root.join("index.json");
        let yaml = root.join(format!("entries/user/{id}.yaml"));
        let target = fixture.path().join("outside");
        let managed = if bypass.starts_with("index") {
            &index
        } else {
            &yaml
        };
        fs::write(&target, fs::read(managed).unwrap()).unwrap();
        fs::remove_file(managed).unwrap();
        if bypass.ends_with("symlink") {
            symlink(&target, managed).unwrap();
        } else {
            fs::hard_link(&target, managed).unwrap();
        }
        let before = fs::read(&target).unwrap();

        let error = Index::load_or_rebuild(&store).unwrap_err();

        assert_eq!(error.code(), "unsafe_store_path", "{bypass}");
        assert_eq!(fs::read(target).unwrap(), before, "{bypass}");
    }
}

#[test]
fn missing_symlinked_and_timed_out_locks_refuse_rebuild_without_yaml_mutation() {
    for bypass in ["missing", "symlink", "timeout"] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        let id = admit_user(
            &store,
            fixture.path(),
            "Lock protected statement.",
            &["lock protected"],
            "Established.",
        );
        let yaml = root.join(format!("entries/user/{id}.yaml"));
        let before = fs::read(&yaml).unwrap();
        fs::write(root.join("index.json"), b"corrupt").unwrap();
        fs::remove_file(root.join(".lock")).unwrap();
        let held = match bypass {
            "missing" => None,
            "symlink" => {
                let outside = fixture.path().join("outside-lock");
                fs::write(&outside, b"unchanged").unwrap();
                symlink(outside, root.join(".lock")).unwrap();
                None
            }
            "timeout" => {
                fs::write(root.join(".lock"), b"").unwrap();
                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(root.join(".lock"))
                    .unwrap();
                rustix::fs::flock(&file, rustix::fs::FlockOperation::LockExclusive).unwrap();
                Some(file)
            }
            _ => unreachable!(),
        };
        let started = Instant::now();

        let error = Index::load_or_rebuild(&store).unwrap_err();

        let expected = if bypass == "timeout" {
            "store_lock_timeout"
        } else {
            "store_lock_unavailable"
        };
        assert_eq!(error.code(), expected, "{bypass}");
        assert_eq!(fs::read(yaml).unwrap(), before, "{bypass}");
        if held.is_some() {
            assert!(started.elapsed() >= Duration::from_secs(2));
            assert!(started.elapsed() < Duration::from_secs(3));
        }
    }
}

#[test]
fn a_replaced_index_temporary_is_never_published() {
    for bypass in ["file", "hardlink", "symlink"] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            memory_root(&root),
            StoreFailpoint::PauseBeforeIndexRename(Arc::clone(&barrier)),
        )
        .unwrap();
        fs::write(root.join("index.json"), b"corrupt").unwrap();
        let root_for_thread = root.clone();
        let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
        barrier.wait();
        let temporary = fs::read_dir(&root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .find(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .contains(".tmp-")
            })
            .unwrap();
        let displaced = temporary.with_extension("displaced");
        fs::rename(&temporary, displaced).unwrap();
        let outside = root_for_thread.with_extension(format!("outside-{bypass}"));
        fs::write(&outside, b"substituted").unwrap();
        match bypass {
            "file" => fs::copy(&outside, &temporary).map(|_| ()).unwrap(),
            "hardlink" => fs::hard_link(&outside, &temporary).unwrap(),
            "symlink" => symlink(&outside, &temporary).unwrap(),
            _ => unreachable!(),
        }
        barrier.wait();

        let error = rebuild.join().unwrap().unwrap_err();

        assert_eq!(error.code(), "unsafe_store_path", "{bypass}");
        assert_eq!(fs::read(outside).unwrap(), b"substituted");
        assert_eq!(fs::read(root.join("index.json")).unwrap(), b"corrupt");
    }
}

#[test]
fn an_index_write_failure_returns_an_error_without_mutating_yaml() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let initial = Store::open(memory_root(&root)).unwrap();
    let id = admit_user(
        &initial,
        fixture.path(),
        "Write failure statement.",
        &["write failure"],
        "Established.",
    );
    let yaml = root.join(format!("entries/user/{id}.yaml"));
    let before = fs::read(&yaml).unwrap();
    fs::write(root.join("index.json"), b"corrupt").unwrap();
    let store =
        Store::open_with_failpoint(memory_root(&root), StoreFailpoint::BeforeIndexWrite).unwrap();

    let error = Index::load_or_rebuild(&store).unwrap_err();

    assert_eq!(error.code(), "store_unavailable");
    assert_eq!(fs::read(yaml).unwrap(), before);
    assert_eq!(fs::read(root.join("index.json")).unwrap(), b"corrupt");
}
