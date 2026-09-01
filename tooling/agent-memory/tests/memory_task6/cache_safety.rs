use super::support::*;
use agent_memory::{
    OracleContext, OracleVerdict, SourceResolution, Store, StoreFailpoint, evaluate_oracle,
};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::sync::{Arc, Barrier};

fn remote_entry() -> agent_memory::MemoryEntry {
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

fn prime(store: &Store, entry: &agent_memory::MemoryEntry) {
    let resolver = FakeResolver::with_responses([valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    assert_eq!(
        evaluate_oracle(
            entry,
            OracleContext::new(store, &clock, &resolver, environment())
        )
        .verdict(),
        OracleVerdict::Valid
    );
}

fn unavailable_after_cache_failure(store: &Store, entry: &agent_memory::MemoryEntry) {
    let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");
    assert_eq!(
        evaluate_oracle(
            entry,
            OracleContext::new(store, &clock, &resolver, environment())
        )
        .verdict(),
        OracleVerdict::Unavailable
    );
}

#[test]
fn corrupt_symlinked_and_hardlinked_caches_never_return_stale_validity() {
    for substitution in ["corrupt", "symlink", "hardlink"] {
        let fixture = tempfile::tempdir().unwrap();
        let (root, store) = open_store(fixture.path());
        let entry = remote_entry();
        prime(&store, &entry);
        let cache = root.join("oracle-cache.json");
        if substitution == "corrupt" {
            fs::write(&cache, b"not json").unwrap();
        } else {
            let displaced = root.join("cache-displaced");
            fs::rename(&cache, &displaced).unwrap();
            let outside = root.join("cache-outside");
            fs::copy(&displaced, &outside).unwrap();
            if substitution == "symlink" {
                symlink(&outside, &cache).unwrap();
            } else {
                fs::hard_link(&outside, &cache).unwrap();
            }
        }

        unavailable_after_cache_failure(&store, &entry);
    }
}

#[test]
fn repairs_cache_mode_before_using_a_fresh_record() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let entry = remote_entry();
    prime(&store, &entry);
    let cache = root.join("oracle-cache.json");
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o644)).unwrap();
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");
    let evaluation = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &clock, &resolver, environment()),
    );

    assert_eq!(evaluation.verdict(), OracleVerdict::Valid);
    assert!(evaluation.from_cache());
    assert_eq!(
        fs::metadata(cache).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn failed_cache_writes_keep_current_validity_but_never_create_a_hit() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store =
        Store::open_with_failpoint(memory_root(&root), StoreFailpoint::BeforeCacheWrite).unwrap();
    let entry = remote_entry();
    let resolver = FakeResolver::with_responses([valid('b'), valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    let first = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &clock, &resolver, environment()),
    );
    let second = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &clock, &resolver, environment()),
    );

    assert_eq!(first.verdict(), OracleVerdict::Valid);
    assert_eq!(second.verdict(), OracleVerdict::Valid);
    assert!(!first.from_cache());
    assert!(!second.from_cache());
    assert_eq!(resolver.calls().len(), 2);
    assert!(cache_json(&root)["entries"].as_array().unwrap().is_empty());
}

#[test]
fn a_deleted_derived_cache_is_rebuilt_after_a_valid_verdict() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    fs::remove_file(root.join("oracle-cache.json")).unwrap();
    let entry = remote_entry();
    let resolver = FakeResolver::with_responses([valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    let first = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &clock, &resolver, environment()),
    );
    assert_eq!(first.verdict(), OracleVerdict::Valid);
    assert!(!first.from_cache());

    let no_refetch = FakeResolver::with_responses([]);
    let second = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &clock, &no_refetch, environment()),
    );
    assert_eq!(second.verdict(), OracleVerdict::Valid);
    assert!(second.from_cache());
    assert!(no_refetch.calls().is_empty());
}

#[test]
fn cache_substitution_after_read_forces_fresh_resolution() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, initial) = open_store(fixture.path());
    let entry = remote_entry();
    prime(&initial, &entry);
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        memory_root(&root),
        StoreFailpoint::PauseAfterCacheRead(Arc::clone(&barrier)),
    )
    .unwrap();
    let worker_barrier = Arc::clone(&barrier);
    let worker = std::thread::spawn(move || {
        let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        let evaluation = evaluate_oracle(
            &entry,
            OracleContext::new(&store, &clock, &resolver, environment()),
        );
        assert_eq!(resolver.calls().len(), 1);
        evaluation.verdict()
    });
    worker_barrier.wait();
    let cache = root.join("oracle-cache.json");
    fs::rename(&cache, root.join("cache-displaced")).unwrap();
    fs::write(&cache, b"not json").unwrap();
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o600)).unwrap();
    worker_barrier.wait();

    assert_eq!(worker.join().unwrap(), OracleVerdict::Unavailable);
}

#[test]
fn cache_substitution_before_rename_preserves_current_verdict_without_publishing() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, _) = open_store(fixture.path());
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        memory_root(&root),
        StoreFailpoint::PauseBeforeCacheRename(Arc::clone(&barrier)),
    )
    .unwrap();
    let worker_barrier = Arc::clone(&barrier);
    let worker = std::thread::spawn(move || {
        let entry = remote_entry();
        let resolver = FakeResolver::with_responses([valid('b')]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        evaluate_oracle(
            &entry,
            OracleContext::new(&store, &clock, &resolver, environment()),
        )
    });
    worker_barrier.wait();
    let cache = root.join("oracle-cache.json");
    fs::rename(&cache, root.join("cache-displaced")).unwrap();
    fs::write(&cache, b"not json").unwrap();
    fs::set_permissions(&cache, fs::Permissions::from_mode(0o600)).unwrap();
    worker_barrier.wait();
    let evaluation = worker.join().unwrap();

    assert_eq!(evaluation.verdict(), OracleVerdict::Valid);
    assert!(!evaluation.from_cache());
    assert_eq!(fs::read(&cache).unwrap(), b"not json");
    let reopened = Store::open(memory_root(&root)).unwrap();
    unavailable_after_cache_failure(&reopened, &remote_entry());
}
