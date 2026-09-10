use super::support::*;
use agent_memory::{
    OracleContext, OracleVerdict, SourceResolution, Store, StoreFailpoint, evaluate_oracle,
};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::{Arc, Barrier};

fn remote_entry() -> FixtureResult<agent_memory::MemoryEntry> {
    entry(
        'b',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/cache-safety",
            fingerprint: 'b',
        }],
    )
}

fn prime(store: &Store, entry: &agent_memory::MemoryEntry) -> FixtureResult<()> {
    let resolver = FakeResolver::with_responses([valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    assert_eq!(
        resolver
            .checked(|checked_resolver| Ok(evaluate_oracle(
                entry,
                &OracleContext::new(store, &clock, checked_resolver, environment())
            )))?
            .verdict(),
        OracleVerdict::Valid
    );
    Ok(())
}

fn unavailable_after_cache_failure(
    store: &Store,
    entry: &agent_memory::MemoryEntry,
) -> FixtureResult<()> {
    let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    assert_eq!(
        resolver
            .checked(|checked_resolver| Ok(evaluate_oracle(
                entry,
                &OracleContext::new(store, &clock, checked_resolver, environment())
            )))?
            .verdict(),
        OracleVerdict::Unavailable
    );
    Ok(())
}

#[test]
fn corrupt_symlinked_and_hardlinked_caches_never_return_stale_validity()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for substitution in ["corrupt", "symlink", "hardlink"] {
        let fixture = tempfile::tempdir()?;
        let (root, store) = open_store(fixture.path())?;
        let entry = remote_entry()?;
        prime(&store, &entry)?;
        let cache = root.join("oracle-cache.json");
        if substitution == "corrupt" {
            fs::write(&cache, b"not json")?;
        } else {
            let displaced = root.join("cache-displaced");
            fs::rename(&cache, &displaced)?;
            let outside = root.join("cache-outside");
            fs::copy(&displaced, &outside)?;
            if substitution == "symlink" {
                symlink(&outside, &cache)?;
            } else {
                fs::hard_link(&outside, &cache)?;
            }
        }

        unavailable_after_cache_failure(&store, &entry)?;
    }
    Ok(())
}

#[test]
fn repairs_cache_mode_before_using_a_fresh_record()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let entry = remote_entry()?;
    prime(&store, &entry)?;
    let cache = root.join("oracle-cache.json");
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o644))?;
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let evaluation = resolver.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert_eq!(evaluation.verdict(), OracleVerdict::Valid);
    assert!(evaluation.from_cache());
    assert_eq!(fs::metadata(cache)?.permissions().mode() & 0o777, 0o600);
    Ok(())
}

#[test]
fn failed_cache_writes_keep_current_validity_but_never_create_a_hit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open_with_failpoint(&memory_root(&root)?, StoreFailpoint::BeforeCacheWrite)?;
    let entry = remote_entry()?;
    let resolver = FakeResolver::with_responses([valid('b'), valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    let first = resolver.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;
    let second = resolver.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert_eq!(first.verdict(), OracleVerdict::Valid);
    assert_eq!(second.verdict(), OracleVerdict::Valid);
    assert!(!first.from_cache());
    assert!(!second.from_cache());
    assert_eq!(resolver.calls().len(), 2);
    assert!(
        cache_json(&root)?
            .get("entries")
            .ok_or("missing fixture element")?
            .as_array()
            .ok_or("missing fixture value")?
            .is_empty()
    );
    Ok(())
}

#[test]
fn a_deleted_derived_cache_is_rebuilt_after_a_valid_verdict()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    fs::remove_file(root.join("oracle-cache.json"))?;
    let entry = remote_entry()?;
    let resolver = FakeResolver::with_responses([valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    let first = resolver.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(first.verdict(), OracleVerdict::Valid);
    assert!(!first.from_cache());

    let no_refetch = FakeResolver::with_responses([]);
    let second = no_refetch.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(second.verdict(), OracleVerdict::Valid);
    assert!(second.from_cache());
    assert!(no_refetch.calls().is_empty());
    Ok(())
}

#[test]
fn cache_substitution_after_read_forces_fresh_resolution()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, initial) = open_store(fixture.path())?;
    let entry = remote_entry()?;
    prime(&initial, &entry)?;
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        &memory_root(&root)?,
        StoreFailpoint::PauseAfterCacheRead(Arc::clone(&barrier)),
    )?;
    let worker_barrier = Arc::clone(&barrier);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let worker = std::thread::spawn(move || {
        let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
        let evaluation = resolver.checked(|checked_resolver| {
            Ok(evaluate_oracle(
                &entry,
                &OracleContext::new(&store, &clock, checked_resolver, environment()),
            ))
        })?;
        assert_eq!(resolver.calls().len(), 1);
        Ok::<_, Box<dyn std::error::Error + Send + Sync>>(evaluation.verdict())
    });
    worker_barrier.wait();
    let cache = root.join("oracle-cache.json");
    fs::rename(&cache, root.join("cache-displaced"))?;
    fs::write(&cache, b"not json")?;
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o600))?;
    worker_barrier.wait();

    assert_eq!(
        worker.join().map_err(|_| "worker thread panicked")??,
        OracleVerdict::Unavailable
    );
    Ok(())
}

#[test]
fn cache_substitution_before_rename_preserves_current_verdict_without_publishing()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, _) = open_store(fixture.path())?;
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        &memory_root(&root)?,
        StoreFailpoint::PauseBeforeCacheRename(Arc::clone(&barrier)),
    )?;
    let worker_barrier = Arc::clone(&barrier);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let entry = remote_entry()?;
    let worker = std::thread::spawn(move || {
        let resolver = FakeResolver::with_responses([valid('b')]);
        resolver.checked(|checked_resolver| {
            Ok(evaluate_oracle(
                &entry,
                &OracleContext::new(&store, &clock, checked_resolver, environment()),
            ))
        })
    });
    worker_barrier.wait();
    let cache = root.join("oracle-cache.json");
    fs::rename(&cache, root.join("cache-displaced"))?;
    fs::write(&cache, b"not json")?;
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o600))?;
    worker_barrier.wait();
    let evaluation = worker.join().map_err(|_| "worker thread panicked")??;

    assert_eq!(evaluation.verdict(), OracleVerdict::Valid);
    assert!(!evaluation.from_cache());
    assert_eq!(fs::read(&cache)?, b"not json");
    let reopened = Store::open(&memory_root(&root)?)?;
    unavailable_after_cache_failure(&reopened, &remote_entry()?)?;
    Ok(())
}
