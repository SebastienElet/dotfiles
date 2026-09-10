use super::support::*;
use agent_memory::{Index, StoreFailpoint};
use std::fs;
use std::os::unix::fs::symlink;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

#[test]
fn index_symlink_hardlink_and_yaml_link_bypasses_fail_closed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for bypass in [
        "index-symlink",
        "index-hardlink",
        "yaml-symlink",
        "yaml-hardlink",
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let store = Store::open(&memory_root(&root)?)?;
        let id = admit_user(
            &store,
            fixture.path(),
            "Linked entry statement.",
            &["linked entry"],
            "Established.",
        )?;
        let index = root.join("index.json");
        let yaml = root.join(format!("entries/user/{id}.yaml"));
        let target = fixture.path().join("outside");
        let managed = if bypass.starts_with("index") {
            &index
        } else {
            &yaml
        };
        fs::write(&target, fs::read(managed)?)?;
        fs::remove_file(managed)?;
        if bypass.ends_with("symlink") {
            symlink(&target, managed)?;
        } else {
            fs::hard_link(&target, managed)?;
        }
        let before = fs::read(&target)?;

        let error = Index::load_or_rebuild(&store)
            .err()
            .ok_or("expected operation failure")?;

        assert_eq!(error.code(), "unsafe_store_path", "{bypass}");
        assert_eq!(fs::read(target)?, before, "{bypass}");
    }
    Ok(())
}

#[test]
fn missing_symlinked_and_timed_out_locks_refuse_rebuild_without_yaml_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for bypass in ["missing", "symlink", "timeout"] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let store = Store::open(&memory_root(&root)?)?;
        let id = admit_user(
            &store,
            fixture.path(),
            "Lock protected statement.",
            &["lock protected"],
            "Established.",
        )?;
        let yaml = root.join(format!("entries/user/{id}.yaml"));
        let before = fs::read(&yaml)?;
        fs::write(root.join("index.json"), b"corrupt")?;
        fs::remove_file(root.join(".lock"))?;
        let held = match bypass {
            "missing" => None,
            "symlink" => {
                let outside = fixture.path().join("outside-lock");
                fs::write(&outside, b"unchanged")?;
                symlink(outside, root.join(".lock"))?;
                None
            }
            "timeout" => {
                fs::write(root.join(".lock"), b"")?;
                let file = fs::OpenOptions::new()
                    .read(true)
                    .write(true)
                    .open(root.join(".lock"))?;
                rustix::fs::flock(&file, rustix::fs::FlockOperation::LockExclusive)?;
                Some(file)
            }
            _ => return Err("unexpected fixture variant".into()),
        };
        let started = Instant::now();

        let error = Index::load_or_rebuild(&store)
            .err()
            .ok_or("expected operation failure")?;

        let expected = if bypass == "timeout" {
            "store_lock_timeout"
        } else {
            "store_lock_unavailable"
        };
        assert_eq!(error.code(), expected, "{bypass}");
        assert_eq!(fs::read(yaml)?, before, "{bypass}");
        if held.is_some() {
            assert!(started.elapsed() >= Duration::from_secs(2));
            assert!(started.elapsed() < Duration::from_secs(3));
        }
    }
    Ok(())
}

#[test]
fn a_replaced_index_temporary_is_never_published()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for bypass in ["file", "hardlink", "symlink"] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let barrier = Arc::new(Barrier::new(2));
        let store = Store::open_with_failpoint(
            &memory_root(&root)?,
            StoreFailpoint::PauseBeforeIndexRename(Arc::clone(&barrier)),
        )?;
        fs::write(root.join("index.json"), b"corrupt")?;
        let root_for_thread = root.clone();
        let rebuild = std::thread::spawn(move || Index::load_or_rebuild(&store));
        barrier.wait();
        let temporary = fs::read_dir(&root)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|entry| entry.path())
            .find(|path| {
                path.file_name()
                    .is_some_and(|name| name.to_string_lossy().contains(".tmp-"))
            })
            .ok_or("missing fixture value")?;
        let displaced = temporary.with_extension("displaced");
        fs::rename(&temporary, displaced)?;
        let outside = root_for_thread.with_extension(format!("outside-{bypass}"));
        fs::write(&outside, b"substituted")?;
        match bypass {
            "file" => fs::copy(&outside, &temporary).map(|_| ())?,
            "hardlink" => fs::hard_link(&outside, &temporary)?,
            "symlink" => symlink(&outside, &temporary)?,
            _ => return Err("unexpected fixture variant".into()),
        }
        barrier.wait();

        let error = rebuild
            .join()
            .map_err(|_| "worker thread panicked")?
            .err()
            .ok_or("expected operation failure")?;

        assert_eq!(error.code(), "unsafe_store_path", "{bypass}");
        assert_eq!(fs::read(outside)?, b"substituted");
        assert_eq!(fs::read(root.join("index.json"))?, b"corrupt");
    }
    Ok(())
}

#[test]
fn an_index_write_failure_returns_an_error_without_mutating_yaml()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let initial = Store::open(&memory_root(&root)?)?;
    let id = admit_user(
        &initial,
        fixture.path(),
        "Write failure statement.",
        &["write failure"],
        "Established.",
    )?;
    let yaml = root.join(format!("entries/user/{id}.yaml"));
    let before = fs::read(&yaml)?;
    fs::write(root.join("index.json"), b"corrupt")?;
    let store = Store::open_with_failpoint(&memory_root(&root)?, StoreFailpoint::BeforeIndexWrite)?;

    let error = Index::load_or_rebuild(&store)
        .err()
        .ok_or("expected operation failure")?;

    assert_eq!(error.code(), "store_unavailable");
    assert_eq!(fs::read(yaml)?, before);
    assert_eq!(fs::read(root.join("index.json"))?, b"corrupt");
    Ok(())
}
