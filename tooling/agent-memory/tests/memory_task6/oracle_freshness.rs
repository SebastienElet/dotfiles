use super::support::*;
use agent_memory::{OracleContext, OracleVerdict, ProofValid, SourceResolution, evaluate_oracle};

#[test]
fn revalidates_local_sources_but_does_not_refetch_a_fresh_url()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (_, store) = open_store(fixture.path())?;
    let local = entry(
        '1',
        "invariant",
        &[SourceFixture {
            kind: "local-file",
            locator: "/tmp/local-proof",
            fingerprint: 'a',
        }],
    )?;
    let initial_local = FakeResolver::with_responses([valid('a')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    assert_eq!(
        initial_local
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &local,
                &OracleContext::new(&store, &clock, checked_resolver, environment())
            )))?
            .verdict(),
        OracleVerdict::Valid
    );
    let changed_local = FakeResolver::with_responses([valid('b')]);
    let later = FixedClock::at("2026-08-28T01:00:00Z")?;
    let answer = ProofValid::new(local.id().as_str())?;
    let changed = changed_local.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &local,
            &OracleContext::new(&store, &later, checked_resolver, environment())
                .with_proof_valid(&answer),
        ))
    })?;
    assert_eq!(changed.verdict(), OracleVerdict::Invalid);

    let remote = entry(
        '2',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/contract?version=1",
            fingerprint: 'c',
        }],
    )?;
    let initial_remote = FakeResolver::with_responses([valid('c')]);
    assert_eq!(
        initial_remote
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &remote,
                &OracleContext::new(&store, &clock, checked_resolver, environment()),
            )))?
            .verdict(),
        agent_memory::OracleVerdict::Valid
    );
    let no_refetch = FakeResolver::with_responses([]);
    let cached = no_refetch.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &remote,
            &OracleContext::new(&store, &later, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(cached.verdict(), OracleVerdict::Valid);
    assert!(cached.from_cache());
    assert!(no_refetch.calls().is_empty());
    Ok(())
}

#[test]
fn expires_a_remote_verdict_and_accepts_only_an_explicit_proof_fallback()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (_, store) = open_store(fixture.path())?;
    let remote = entry(
        '3',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/contract",
            fingerprint: 'd',
        }],
    )?;
    let initial = FakeResolver::with_responses([valid('d')]);
    let validated = FixedClock::at("2026-08-28T00:00:00Z")?;
    assert_eq!(
        initial
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &remote,
                &OracleContext::new(&store, &validated, checked_resolver, environment()),
            )))?
            .verdict(),
        agent_memory::OracleVerdict::Valid
    );
    let expired = FixedClock::at("2026-08-30T00:00:00Z")?;
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let omitted = unavailable.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &remote,
            &OracleContext::new(&store, &expired, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(omitted.verdict(), OracleVerdict::Unavailable);

    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let wrong_answer = ProofValid::new("mem_ffffffffffffffffffffffff")?;
    let still_omitted = unavailable.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &remote,
            &OracleContext::new(&store, &expired, checked_resolver, environment())
                .with_proof_valid(&wrong_answer),
        ))
    })?;
    assert_eq!(still_omitted.verdict(), OracleVerdict::Unavailable);

    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let answer = ProofValid::new(remote.id().as_str())?;
    let fallback = unavailable.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &remote,
            &OracleContext::new(&store, &expired, checked_resolver, environment())
                .with_proof_valid(&answer),
        ))
    })?;
    assert_eq!(fallback.verdict(), OracleVerdict::Valid);
    assert!(!fallback.from_cache());
    Ok(())
}

#[test]
fn proof_valid_refuses_an_invalid_entry_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    assert_eq!(
        ProofValid::new("not-a-memory-id")
            .err()
            .ok_or("expected operation failure")?
            .code(),
        "invalid_memory_id"
    );
    Ok(())
}

#[test]
fn a_changed_remote_locator_cannot_reuse_a_matching_cached_fingerprint()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (_, store) = open_store(fixture.path())?;
    let original = entry(
        '4',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/original",
            fingerprint: 'e',
        }],
    )?;
    let initial = FakeResolver::with_responses([valid('e')]);
    let validated = FixedClock::at("2026-08-28T00:00:00Z")?;
    assert_eq!(
        initial
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &original,
                &OracleContext::new(&store, &validated, checked_resolver, environment()),
            )))?
            .verdict(),
        agent_memory::OracleVerdict::Valid
    );
    let changed = entry(
        '4',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/replacement",
            fingerprint: 'e',
        }],
    )?;
    let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let later = FixedClock::at("2026-08-28T01:00:00Z")?;
    let evaluation = resolver.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &changed,
            &OracleContext::new(&store, &later, checked_resolver, environment()),
        ))
    })?;

    assert_eq!(evaluation.verdict(), OracleVerdict::Unavailable);
    assert_eq!(
        resolver.calls().first().ok_or("missing fixture element")?.1,
        "https://docs.example.test/replacement"
    );
    Ok(())
}
