use super::support::*;
use agent_memory::{
    ProofAnswers, ProofValid, RetrievalContext, RetrievalRequest, Status, retrieve,
};

#[test]
fn proof_valid_is_cached_without_creating_a_transition() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let yaml = entry_yaml(
        'c',
        "invariant",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:fallback",
            fingerprint: 'c',
        }],
    );
    write_user_entry(&root, 'c', &yaml);
    let key = project_key(fixture.path());
    let selection = select(&store, &key, 5);
    let id = user_entry_id('c', "invariant");
    let mut answers = ProofAnswers::new();
    answers.insert(ProofValid::new(&id).unwrap());
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z");
    let report = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(&store, &clock, &resolver, environment())
            .with_proof_answers(&answers),
    );

    assert_eq!(report.injected.len(), 1);
    let cached = retrieve(
        RetrievalRequest::new(&selection, &key, true),
        RetrievalContext::new(
            &store,
            &FixedClock::at("2026-08-28T02:00:00Z"),
            &FakeResolver::with_responses([]),
            environment(),
        ),
    );
    assert_eq!(cached.injected[0].verdict_age_milliseconds, 3_600_000);
    let stored = store.load(&id).unwrap().unwrap();
    assert_eq!(stored.status(), Status::Active);
    assert!(stored.transition().is_none());
}
