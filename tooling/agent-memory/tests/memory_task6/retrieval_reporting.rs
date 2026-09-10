use super::support::*;
use agent_memory::{
    Clock, OmissionEffect, RetrievalContext, RetrievalRequest, SourceResolution, UtcTimestamp,
    retrieve,
};
use std::sync::atomic::{AtomicUsize, Ordering};

struct SequencedClock {
    timestamps: Vec<UtcTimestamp>,
    final_timestamp: UtcTimestamp,
    reads: AtomicUsize,
}

impl SequencedClock {
    fn new(timestamps: impl IntoIterator<Item = &'static str>) -> FixtureResult<Self> {
        let timestamps = timestamps
            .into_iter()
            .map(agent_memory::parse_utc_timestamp)
            .collect::<Result<Vec<_>, _>>()?;
        let final_timestamp = timestamps
            .last()
            .ok_or("a sequenced clock needs at least one timestamp")?
            .clone();
        Ok(Self {
            timestamps,
            final_timestamp,
            reads: AtomicUsize::new(0),
        })
    }

    fn remaining(&self) -> FixtureResult<usize> {
        self.timestamps
            .len()
            .checked_sub(self.reads.load(Ordering::Acquire))
            .ok_or_else(|| "unexpected clock read exhausted the fixture".into())
    }
}

impl Clock for SequencedClock {
    fn now(&self) -> UtcTimestamp {
        let index = self.reads.fetch_add(1, Ordering::AcqRel);
        self.timestamps
            .get(index)
            .unwrap_or(&self.final_timestamp)
            .clone()
    }
}

#[test]
fn reports_the_age_of_the_cached_valid_verdict()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        'e',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/age",
            fingerprint: 'e',
        }],
    )?;
    write_user_entry(&root, 'e', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 5)?;
    let resolver = FakeResolver::with_responses([valid('e')]);
    let validated = FixedClock::at("2026-08-28T01:00:00Z")?;
    let first = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &validated, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(
        first
            .injected
            .first()
            .ok_or("missing fixture element")?
            .verdict_age_milliseconds,
        0
    );

    let no_refetch = FakeResolver::with_responses([]);
    let later = FixedClock::at("2026-08-28T02:00:00Z")?;
    let second = no_refetch.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &later, checked_resolver, environment()),
        ))
    })?;
    assert_eq!(
        second
            .injected
            .first()
            .ok_or("missing fixture element")?
            .verdict_age_milliseconds,
        3_600_000
    );
    assert!(no_refetch.calls().is_empty());
    Ok(())
}

#[test]
fn carries_one_freshness_snapshot_through_retrieval_at_the_48_hour_boundary()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        'a',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/freshness-snapshot",
            fingerprint: 'a',
        }],
    )?;
    write_user_entry(&root, 'a', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 5)?;
    let resolver = FakeResolver::with_responses([valid('a')]);
    assert_eq!(
        resolver
            .checked(|checked_resolver| Ok(retrieve(
                RetrievalRequest::new(&selection, &key, true),
                &RetrievalContext::new(
                    &store,
                    &FixedClock::at("2026-08-28T00:00:00Z")?,
                    checked_resolver,
                    environment(),
                ),
            )))?
            .injected
            .len(),
        1
    );
    let clock = SequencedClock::new(["2026-08-29T23:59:59.999Z", "2026-08-30T00:00:00Z"])?;
    let no_refetch = FakeResolver::with_responses([]);

    let report = no_refetch.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, true),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert_eq!(
        report
            .injected
            .first()
            .ok_or("missing fixture element")?
            .verdict_age_milliseconds,
        172_799_999
    );
    assert_eq!(clock.remaining()?, 1);
    assert!(no_refetch.calls().is_empty());
    Ok(())
}

#[test]
fn revalidates_scope_after_selection_and_uses_the_exact_omission_effect()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let yaml = entry_yaml(
        'f',
        "invariant",
        &[SourceFixture {
            kind: "official-url",
            locator: "https://docs.example.test/scope",
            fingerprint: 'f',
        }],
    )?;
    write_user_entry(&root, 'f', &yaml)?;
    let key = project_key(fixture.path())?;
    let selection = select(&store, &key, 5)?;
    let resolver = FakeResolver::with_responses([SourceResolution::Unavailable]);
    let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
    let report = resolver.checked(|checked_resolver| {
        Ok(retrieve(
            RetrievalRequest::new(&selection, &key, false),
            &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
        ))
    })?;

    assert!(report.injected.is_empty());
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .code,
        "selection_stale"
    );
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .effect,
        OmissionEffect::NotApplied
    );
    assert_eq!(
        report
            .omitted
            .first()
            .ok_or("missing fixture element")?
            .effect
            .as_str(),
        "not_applied"
    );
    assert!(resolver.calls().is_empty());
    Ok(())
}
