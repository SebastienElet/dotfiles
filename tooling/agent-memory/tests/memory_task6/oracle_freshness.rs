use super::support::*;
use agent_memory::{OracleContext, OracleVerdict, SourceResolution, evaluate_oracle};

#[test]
fn revalidates_local_sources_but_does_not_refetch_a_fresh_url() {
    let fixture = tempfile::tempdir().unwrap();
    let (_, store) = open_store(fixture.path());
    let local = entry(
        '1',
        "invariant",
        &[SourceFixture {
            kind: "local-file",
            locator: "/tmp/local-proof",
            fingerprint: 'a',
        }],
    );
    let initial_local = FakeResolver::with_responses([valid('a')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    assert_eq!(
        evaluate_oracle(
            &local,
            OracleContext::new(&store, &clock, &initial_local, environment())
        )
        .verdict(),
        OracleVerdict::Valid
    );
    let changed_local = FakeResolver::with_responses([valid('b')]);
    let later = FixedClock::at("2026-08-28T01:00:00Z");
    let changed = evaluate_oracle(
        &local,
        OracleContext::new(&store, &later, &changed_local, environment()),
    );
    assert_eq!(changed.verdict(), OracleVerdict::Invalid);

    let remote = entry(
        '2',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/contract?version=1",
            fingerprint: 'c',
        }],
    );
    let initial_remote = FakeResolver::with_responses([valid('c')]);
    evaluate_oracle(
        &remote,
        OracleContext::new(&store, &clock, &initial_remote, environment()),
    );
    let no_refetch = FakeResolver::with_responses([]);
    let cached = evaluate_oracle(
        &remote,
        OracleContext::new(&store, &later, &no_refetch, environment()),
    );
    assert_eq!(cached.verdict(), OracleVerdict::Valid);
    assert!(cached.from_cache());
    assert!(no_refetch.calls().is_empty());
}

#[test]
fn expires_a_remote_verdict_and_accepts_only_an_explicit_proof_fallback() {
    let fixture = tempfile::tempdir().unwrap();
    let (_, store) = open_store(fixture.path());
    let remote = entry(
        '3',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/contract",
            fingerprint: 'd',
        }],
    );
    let initial = FakeResolver::with_responses([valid('d')]);
    let validated = FixedClock::at("2026-08-28T00:00:00Z");
    evaluate_oracle(
        &remote,
        OracleContext::new(&store, &validated, &initial, environment()),
    );
    let expired = FixedClock::at("2026-08-30T00:00:00Z");
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let omitted = evaluate_oracle(
        &remote,
        OracleContext::new(&store, &expired, &unavailable, environment()),
    );
    assert_eq!(omitted.verdict(), OracleVerdict::Unavailable);

    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let fallback = evaluate_oracle(
        &remote,
        OracleContext::new(&store, &expired, &unavailable, environment()).with_proof_valid(),
    );
    assert_eq!(fallback.verdict(), OracleVerdict::Valid);
    assert!(!fallback.from_cache());
}

#[test]
fn a_changed_remote_locator_cannot_reuse_a_matching_cached_fingerprint() {
    let fixture = tempfile::tempdir().unwrap();
    let (_, store) = open_store(fixture.path());
    let original = entry(
        '4',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/original",
            fingerprint: 'e',
        }],
    );
    let initial = FakeResolver::with_responses([valid('e')]);
    let validated = FixedClock::at("2026-08-28T00:00:00Z");
    evaluate_oracle(
        &original,
        OracleContext::new(&store, &validated, &initial, environment()),
    );
    let changed = entry(
        '4',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/replacement",
            fingerprint: 'e',
        }],
    );
    let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let later = FixedClock::at("2026-08-28T01:00:00Z");
    let evaluation = evaluate_oracle(
        &changed,
        OracleContext::new(&store, &later, &resolver, environment()),
    );

    assert_eq!(evaluation.verdict(), OracleVerdict::Unavailable);
    assert_eq!(
        resolver.calls()[0].1,
        "https://docs.example.test/replacement"
    );
}
