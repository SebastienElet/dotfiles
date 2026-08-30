use super::support::*;
use agent_memory::{
    MemoryErrorClass, RetrievalContext, RetrievalRequest, SourceResolution, SourceResolver,
    retrieve_for_injection,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

struct DeadlineAfterUnavailableResolver {
    expired: AtomicBool,
    calls: AtomicUsize,
}

impl SourceResolver for DeadlineAfterUnavailableResolver {
    fn resolve(&self, _source: &agent_memory::EntrySource) -> SourceResolution {
        self.calls.fetch_add(1, Ordering::AcqRel);
        self.expired.store(true, Ordering::Release);
        SourceResolution::Unavailable
    }

    fn deadline_exceeded(&self) -> bool {
        self.expired.load(Ordering::Acquire)
    }
}

#[test]
fn an_expired_source_budget_stops_before_later_source_resolution() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        'e',
        "invariant",
        &[
            SourceFixture {
                kind: "local-file",
                locator: "/tmp/first-proof",
                fingerprint: 'a',
            },
            SourceFixture {
                kind: "local-file",
                locator: "/tmp/second-proof",
                fingerprint: 'a',
            },
        ],
    );
    write_user_entry(&root, 'e', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let resolver = DeadlineAfterUnavailableResolver {
        expired: AtomicBool::new(false),
        calls: AtomicUsize::new(0),
    };

    let error = retrieve_for_injection(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(
            &store,
            &FixedClock::at("2026-08-28T01:00:00Z"),
            &resolver,
            environment(),
        ),
    )
    .unwrap_err();

    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(resolver.calls.load(Ordering::Acquire), 1);
}
