use super::support::*;
use agent_memory::{OracleContext, OracleVerdict, SourceResolution, evaluate_oracle};
use std::fs;

fn remote_entry() -> FixtureResult<agent_memory::MemoryEntry> {
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

fn prime(store: &agent_memory::Store, entry: &agent_memory::MemoryEntry) -> FixtureResult<()> {
    let resolver = FakeResolver::with_responses([valid('a')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    let evaluation = resolver.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            entry,
            &OracleContext::new(store, &clock, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(evaluation.verdict(), OracleVerdict::Valid);
    Ok(())
}

#[test]
fn cache_is_usable_strictly_before_48_hours_but_not_at_the_boundary()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (_, store) = open_store(fixture.path())?;
    let entry = remote_entry()?;
    prime(&store, &entry)?;

    let fresh = FixedClock::at("2026-08-29T23:59:59.999999999Z")?;
    let no_refetch = FakeResolver::with_responses([]);
    let hit = no_refetch.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &fresh, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(hit.verdict(), OracleVerdict::Valid);
    assert!(hit.from_cache());

    let expired = FixedClock::at("2026-08-30T00:00:00Z")?;
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let miss = unavailable.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &expired, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(miss.verdict(), OracleVerdict::Unavailable);
    assert!(!miss.from_cache());
    Ok(())
}

#[test]
fn cache_timestamp_in_the_future_is_always_a_miss()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (_, store) = open_store(fixture.path())?;
    let entry = remote_entry()?;
    prime(&store, &entry)?;
    let before_validation = FixedClock::at("2026-08-27T23:59:59.999999999Z")?;
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let evaluation = unavailable.checked(|checked_resolver| {
        Ok(evaluate_oracle(
            &entry,
            &OracleContext::new(&store, &before_validation, checked_resolver, environment()),
        ))
    })?;

    assert_eq!(evaluation.verdict(), OracleVerdict::Unavailable);
    assert!(!evaluation.from_cache());
    Ok(())
}

#[test]
fn malformed_timestamps_and_non_valid_verdicts_are_cache_misses()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (field, replacement) in [("validated_at", "not-a-timestamp"), ("verdict", "invalid")] {
        let fixture = tempfile::tempdir()?;
        let (root, store) = open_store(fixture.path())?;
        let entry = remote_entry()?;
        prime(&store, &entry)?;
        let mut cache = cache_json(&root)?;
        *cache
            .get_mut("entries")
            .ok_or("missing fixture element")?
            .get_mut(0)
            .ok_or("missing fixture element")?
            .get_mut(field)
            .ok_or("missing fixture element")? = replacement.into();
        fs::write(
            root.join("oracle-cache.json"),
            serde_json::to_vec_pretty(&cache)?,
        )?;
        let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
        let evaluation = resolver.checked(|checked_resolver| {
            Ok(evaluate_oracle(
                &entry,
                &OracleContext::new(&store, &clock, checked_resolver, environment()),
            ))
        })?;

        assert_eq!(evaluation.verdict(), OracleVerdict::Unavailable, "{field}");
        assert!(!evaluation.from_cache(), "{field}");
    }
    Ok(())
}

#[test]
fn cache_records_are_sorted_by_entry_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    for id in ['f', '1', '9'] {
        let entry = entry(
            id,
            "invariant",
            &[SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/cache",
                fingerprint: id,
            }],
        )?;
        let resolver = FakeResolver::with_responses([valid(id)]);
        assert_eq!(
            resolver
                .checked(|checked_resolver| Ok(evaluate_oracle(
                    &entry,
                    &OracleContext::new(&store, &clock, checked_resolver, environment()),
                )))?
                .verdict(),
            agent_memory::OracleVerdict::Valid
        );
    }
    let cache = cache_json(&root)?;
    let ids = cache
        .get("entries")
        .ok_or("missing fixture element")?
        .as_array()
        .ok_or("missing fixture value")?
        .iter()
        .map(|entry| entry["entry_id"].as_str().ok_or("missing fixture value"))
        .collect::<Result<Vec<_>, _>>()?;

    let mut expected = ['1', '9', 'f']
        .map(|id| user_entry_id(id, "invariant"))
        .to_vec();
    expected.sort();
    assert_eq!(ids, expected);
    Ok(())
}
