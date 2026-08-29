use super::support::*;
use agent_memory::{
    Clock, OmissionEffect, RetrievalContext, RetrievalRequest, SourceResolution, UtcTimestamp,
    retrieve,
};
use std::collections::VecDeque;
use std::sync::Mutex;

struct SequencedClock {
    timestamps: Mutex<VecDeque<UtcTimestamp>>,
}

impl SequencedClock {
    fn new(timestamps: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            timestamps: Mutex::new(
                timestamps
                    .into_iter()
                    .map(|timestamp| agent_memory::parse_utc_timestamp(timestamp).unwrap())
                    .collect(),
            ),
        }
    }

    fn remaining(&self) -> usize {
        self.timestamps.lock().unwrap().len()
    }
}

impl Clock for SequencedClock {
    fn now(&self) -> UtcTimestamp {
        self.timestamps
            .lock()
            .unwrap()
            .pop_front()
            .expect("unexpected clock read")
    }
}

#[test]
fn reports_the_age_of_the_cached_valid_verdict() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        'e',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/age",
            fingerprint: 'e',
        }],
    );
    write_user_entry(&root, 'e', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let resolver = FakeResolver::with_responses([valid('e')]);
    let validated = FixedClock::at("2026-08-28T01:00:00Z");
    let first = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &validated, &resolver, environment()),
    );
    assert_eq!(first.injected[0].verdict_age_milliseconds, 0);

    let no_refetch = FakeResolver::with_responses([]);
    let later = FixedClock::at("2026-08-28T02:00:00Z");
    let second = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &later, &no_refetch, environment()),
    );
    assert_eq!(second.injected[0].verdict_age_milliseconds, 3_600_000);
    assert!(no_refetch.calls().is_empty());
}

#[test]
fn carries_one_freshness_snapshot_through_retrieval_at_the_48_hour_boundary() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        'a',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/freshness-snapshot",
            fingerprint: 'a',
        }],
    );
    write_user_entry(&root, 'a', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let resolver = FakeResolver::with_responses([valid('a')]);
    retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(
            &store,
            &FixedClock::at("2026-08-28T00:00:00Z"),
            &resolver,
            environment(),
        ),
    );
    let clock = SequencedClock::new(["2026-08-29T23:59:59.999Z", "2026-08-30T00:00:00Z"]);
    let no_refetch = FakeResolver::with_responses([]);

    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &no_refetch, environment()),
    );

    assert_eq!(report.injected[0].verdict_age_milliseconds, 172_799_999);
    assert_eq!(clock.remaining(), 1);
    assert!(no_refetch.calls().is_empty());
}

#[test]
fn revalidates_scope_after_selection_and_uses_the_exact_omission_effect() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        'f',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/scope",
            fingerprint: 'f',
        }],
    );
    write_user_entry(&root, 'f', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");
    let report = retrieve(
        RetrievalRequest::new(&selection, &key, false),
        RetrievalContext::new(&store, &clock, &resolver, environment()),
    );

    assert!(report.injected.is_empty());
    assert_eq!(report.omitted[0].code, "selection_stale");
    assert_eq!(report.omitted[0].effect, OmissionEffect::NotApplied);
    assert_eq!(report.omitted[0].effect.as_str(), "not_applied");
    assert!(resolver.calls().is_empty());
}
