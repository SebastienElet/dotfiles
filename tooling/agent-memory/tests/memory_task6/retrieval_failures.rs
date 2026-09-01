use super::support::*;
use agent_memory::{
    MemoryErrorClass, RetrievalContext, RetrievalRequest, SourceResolution, SourceResolver, Store,
    StoreFailpoint, retrieve, retrieve_for_injection,
};
use std::fs;
use std::os::unix::fs::symlink;
use std::sync::{Arc, Barrier};
use std::time::{Duration, Instant};

struct PausingResolver {
    started: Arc<Barrier>,
    resume: Arc<Barrier>,
}

impl SourceResolver for PausingResolver {
    fn resolve(&self, _source: &agent_memory::EntrySource) -> SourceResolution {
        self.started.wait();
        self.resume.wait();
        valid('a')
    }
}

#[test]
fn unavailable_initial_load_is_fatal_for_injection_but_remains_a_direct_omission() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = local_entry('1');
    let path = write_user_entry(&root, '1', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    replace_with_symlink(&path, &root.join("displaced-initial"));
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");

    let error = retrieve_for_injection(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment()),
    )
    .unwrap_err();

    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(error.code(), "unsafe_store_path");
    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment()),
    );
    assert!(report.injected.is_empty());
    assert_eq!(report.omitted[0].code, "unsafe_store_path");
    assert_eq!(
        serde_json::to_value(report).unwrap(),
        serde_json::json!({
            "injected": [],
            "omitted": [{
                "id": user_entry_id('1', "invariant"),
                "code": "unsafe_store_path",
                "question": null,
                "effect": "not_applied",
            }],
            "omitted_by_limit": 0,
        })
    );
}

#[test]
fn one_expired_hook_deadline_stops_before_loading_selected_entries() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    write_user_entry(&root, '4', &local_entry('4'));
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");

    let error = retrieve_for_injection(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment())
            .with_deadline(Instant::now() - Duration::from_millis(1)),
    )
    .unwrap_err();

    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(error.code(), "retrieval_deadline_exceeded");
}

#[test]
fn unavailable_oracle_is_fatal_for_injection_without_code_heuristics() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    write_user_entry(&root, '5', &local_entry('5'));
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");

    let error = retrieve_for_injection(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment()),
    )
    .unwrap_err();

    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(error.code(), "oracle_unavailable");
}

#[test]
fn unavailable_pre_injection_revalidation_is_fatal() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = local_entry('2');
    let path = write_user_entry(&root, '2', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let started = Arc::new(Barrier::new(2));
    let resume = Arc::new(Barrier::new(2));
    let resolver = PausingResolver {
        started: Arc::clone(&started),
        resume: Arc::clone(&resume),
    };
    let retrieval = std::thread::spawn(move || {
        retrieve_for_injection(
            RetrievalRequest::new(&selection, &key, true),
            RetrievalContext::new(
                &store,
                &FixedClock::at("2026-08-28T01:00:00Z"),
                &resolver,
                environment(),
            ),
        )
    });
    started.wait();
    replace_with_symlink(&path, &root.join("displaced-revalidation"));
    resume.wait();

    let error = retrieval.join().unwrap().unwrap_err();
    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(error.code(), "unsafe_store_path");
}

#[test]
fn unavailable_invalidation_write_is_fatal() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, initial) = open_store(fixture.path());
    let yaml = local_entry('3');
    write_user_entry(&root, '3', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&initial, &key, 5);
    let barrier = Arc::new(Barrier::new(2));
    let store = Store::open_with_failpoint(
        memory_root(&root),
        StoreFailpoint::PauseBeforeYamlRename(Arc::clone(&barrier)),
    )
    .unwrap();
    let retrieval = std::thread::spawn(move || {
        retrieve_for_injection(
            RetrievalRequest::new(&selection, &key, true),
            RetrievalContext::new(
                &store,
                &FixedClock::at("2026-08-28T01:00:00Z"),
                &FakeResolver::with_responses([valid('b')]),
                environment(),
            ),
        )
    });
    barrier.wait();
    let temporary = fs::read_dir(root.join("entries/user"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.to_string_lossy().contains(".tmp-"))
        .unwrap();
    replace_with_symlink(&temporary, &root.join("displaced-invalidation"));
    barrier.wait();

    let error = retrieval.join().unwrap().unwrap_err();
    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(error.code(), "unsafe_store_path");
}

fn local_entry(id: char) -> Vec<u8> {
    entry_yaml(
        id,
        "invariant",
        &[SourceFixture {
            kind: "local-file",
            locator: "/tmp/durable-proof",
            fingerprint: 'a',
        }],
    )
}

fn replace_with_symlink(path: &std::path::Path, displaced: &std::path::Path) {
    fs::rename(path, displaced).unwrap();
    symlink(displaced, path).unwrap();
}
