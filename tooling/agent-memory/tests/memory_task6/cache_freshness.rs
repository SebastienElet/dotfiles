use super::support::*;
use agent_memory::{OracleContext, OracleVerdict, SourceResolution, evaluate_oracle};

fn remote_entry() -> agent_memory::MemoryEntry {
    entry(
        'a',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/cache",
            fingerprint: 'a',
        }],
    )
}

fn prime(store: &agent_memory::Store, entry: &agent_memory::MemoryEntry) {
    let resolver = FakeResolver::with_responses([valid('a')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    let evaluation = evaluate_oracle(
        entry,
        OracleContext::new(store, &clock, &resolver, environment()),
    );
    assert_eq!(evaluation.verdict(), OracleVerdict::Valid);
}

#[test]
fn cache_is_usable_strictly_before_48_hours_but_not_at_the_boundary() {
    let fixture = tempfile::tempdir().unwrap();
    let (_, store) = open_store(fixture.path());
    let entry = remote_entry();
    prime(&store, &entry);

    let fresh = FixedClock::at("2026-08-29T23:59:59.999Z");
    let no_refetch = FakeResolver::with_responses([]);
    let hit = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &fresh, &no_refetch, environment()),
    );
    assert_eq!(hit.verdict(), OracleVerdict::Valid);
    assert!(hit.from_cache());

    let expired = FixedClock::at("2026-08-30T00:00:00Z");
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let miss = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &expired, &unavailable, environment()),
    );
    assert_eq!(miss.verdict(), OracleVerdict::Unavailable);
    assert!(!miss.from_cache());
}

#[test]
fn cache_timestamp_in_the_future_is_always_a_miss() {
    let fixture = tempfile::tempdir().unwrap();
    let (_, store) = open_store(fixture.path());
    let entry = remote_entry();
    prime(&store, &entry);
    let before_validation = FixedClock::at("2026-08-27T23:59:59.999Z");
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let evaluation = evaluate_oracle(
        &entry,
        OracleContext::new(&store, &before_validation, &unavailable, environment()),
    );

    assert_eq!(evaluation.verdict(), OracleVerdict::Unavailable);
    assert!(!evaluation.from_cache());
}

#[test]
fn cache_records_are_sorted_by_entry_id() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    for id in ['f', '1', '9'] {
        let entry = entry(
            id,
            "invariant",
            &[SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/cache",
                fingerprint: id,
            }],
        );
        let resolver = FakeResolver::with_responses([valid(id)]);
        evaluate_oracle(
            &entry,
            OracleContext::new(&store, &clock, &resolver, environment()),
        );
    }
    let cache = cache_json(&root);
    let ids = cache["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["entry_id"].as_str().unwrap())
        .collect::<Vec<_>>();

    assert_eq!(
        ids,
        [
            "mem_111111111111111111111111",
            "mem_999999999999999999999999",
            "mem_ffffffffffffffffffffffff"
        ]
    );
}
