use super::support::*;
use agent_memory::{
    OmissionEffect, RetrievalContext, RetrievalRequest, SourceResolution, retrieve,
};

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
