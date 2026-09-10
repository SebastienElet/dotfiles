use super::support::*;
use agent_memory::{
    ProofAnswers, ProofValid, RetrievalContext, RetrievalRequest, Status, retrieve,
};

#[test]
fn proof_valid_is_cached_without_creating_a_transition()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        'c',
        "invariant",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:fallback",
            fingerprint: 'c',
        }],
    )?;
    write_user_entry(&root, 'c', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 5)?;
    let id = user_entry_id('c', "invariant");
    let mut answers = ProofAnswers::new();
    answers.insert(ProofValid::new(&id)?);
    let resolver = FakeResolver::with_responses([]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let report = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment())
                .with_proof_answers(&answers),
        ))
    })?;

    assert_eq!(report.injected.len(), 1);
    let cached = FakeResolver::with_responses([]).checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(
                &store,
                &FixedClock::at("2026-08-28T02:00:00Z")?,
                checked_resolver,
                environment(),
            ),
        ))
    })?;
    assert_eq!(
        cached
            .injected
            .first()
            .ok_or("missing fixture element")?
            .verdict_age_milliseconds,
        3_600_000
    );
    let stored = store.load(&id)?.ok_or("missing fixture value")?;
    assert_eq!(stored.status(), Status::Active);
    assert!(stored.transition().is_none());
    Ok(())
}
