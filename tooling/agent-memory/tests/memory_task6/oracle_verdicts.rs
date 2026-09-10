use super::support::*;
use agent_memory::{OracleContext, OracleVerdict, SourceResolution, evaluate_oracle};

#[test]
fn persists_only_valid_verdicts_with_canonical_ordered_cache_fields()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let valid_entry = entry(
        '5',
        "invariant",
        &[
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/first",
                fingerprint: 'a',
            },
            SourceFixture {
                kind: "local-file",
                locator: "/tmp/second",
                fingerprint: 'b',
            },
        ],
    )?;
    let resolver = FakeResolver::with_responses([valid('a'), valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    assert_eq!(
        resolver
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &valid_entry,
                &OracleContext::new(&store, &clock, checked_resolver, environment()),
            )))?
            .verdict(),
        agent_memory::OracleVerdict::Valid
    );

    let cache = cache_json(&root)?;
    assert_eq!(
        cache
            .get("schema_version")
            .ok_or("missing fixture element")?,
        1
    );
    let record = &cache
        .get("entries")
        .ok_or("missing fixture element")?
        .get(0)
        .ok_or("missing fixture element")?;
    assert_eq!(record["entry_id"], user_entry_id('5', "invariant"));
    assert_eq!(record["verdict"], "valid");
    assert_eq!(record["validated_at"], "2026-08-28T00:00:00Z");
    assert_eq!(
        record["environment"]
            .get("os")
            .ok_or("missing fixture element")?,
        "macos"
    );
    assert_eq!(
        record["environment"]
            .get("arch")
            .ok_or("missing fixture element")?,
        "aarch64"
    );
    assert_eq!(
        record["source_fingerprints"]
            .get(0)
            .ok_or("missing fixture element")?
            .get("kind")
            .ok_or("missing fixture element")?,
        "official-url"
    );
    assert_eq!(
        record["source_fingerprints"]
            .get(0)
            .ok_or("missing fixture element")?
            .get("fingerprint")
            .ok_or("missing fixture element")?,
        &fingerprint('a')
    );
    assert_eq!(
        record["source_fingerprints"]
            .get(1)
            .ok_or("missing fixture element")?
            .get("kind")
            .ok_or("missing fixture element")?,
        "local-file"
    );
    assert_eq!(
        record["source_fingerprints"]
            .get(1)
            .ok_or("missing fixture element")?
            .get("fingerprint")
            .ok_or("missing fixture element")?,
        &fingerprint('b')
    );
    assert_eq!(
        record["oracle_digest"],
        "sha256:92c5240dc81207720563324941353993080e84180b689efc52af46840add96c9"
    );
    assert_eq!(
        record["proof_digest"],
        "sha256:625882fbac49144bea8bd2b2d9079b287544e53a02cab481bc49a4d199bff679"
    );
    let serialized = serde_json::to_string(&cache)?;
    assert!(!serialized.contains("https://docs.example.test/first"));
    assert!(!serialized.contains("/tmp/second"));
    Ok(())
}

#[test]
fn never_caches_invalid_unavailable_or_needs_confirmation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let automated = |id| {
        entry(
            id,
            "invariant",
            &[SourceFixture {
                kind: "local-file",
                locator: "/tmp/proof",
                fingerprint: 'a',
            }],
        )
    };
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    let invalid = FakeResolver::with_responses([valid('b')]);
    assert_eq!(
        invalid
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &automated('6')?,
                &OracleContext::new(&store, &clock, checked_resolver, environment())
            )))?
            .verdict(),
        OracleVerdict::Invalid
    );
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    assert_eq!(
        unavailable
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &automated('7')?,
                &OracleContext::new(&store, &clock, checked_resolver, environment())
            )))?
            .verdict(),
        OracleVerdict::Unavailable
    );
    let decision = entry(
        '8',
        "invariant",
        &[SourceFixture {
            kind: "user-decision",
            locator: "decision:test",
            fingerprint: 'c',
        }],
    )?;
    let resolver = FakeResolver::with_responses([]);
    assert_eq!(
        resolver
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &decision,
                &OracleContext::new(&store, &clock, checked_resolver, environment())
            )))?
            .verdict(),
        OracleVerdict::NeedsConfirmation
    );

    assert!(
        cache_json(&root)?
            .get("entries")
            .ok_or("missing fixture element")?
            .as_array()
            .ok_or("missing fixture value")?
            .is_empty()
    );
    Ok(())
}

#[test]
fn environment_and_source_order_are_part_of_cache_identity()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (_, store) = open_store(fixture.path())?;
    let original = entry(
        '9',
        "invariant",
        &[
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/a",
                fingerprint: 'a',
            },
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/b",
                fingerprint: 'b',
            },
        ],
    )?;
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    let initial = FakeResolver::with_responses([valid('a'), valid('b')]);
    assert_eq!(
        initial
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &original,
                &OracleContext::new(&store, &clock, checked_resolver, environment()),
            )))?
            .verdict(),
        agent_memory::OracleVerdict::Valid
    );
    let reversed = entry(
        '9',
        "invariant",
        &[
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/b",
                fingerprint: 'b',
            },
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/a",
                fingerprint: 'a',
            },
        ],
    )?;
    let changed_order = FakeResolver::with_responses([
        SourceResolution::Unavailable,
        SourceResolution::Unavailable,
    ]);
    assert_eq!(
        changed_order
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &reversed,
                &OracleContext::new(&store, &clock, checked_resolver, environment())
            )))?
            .verdict(),
        OracleVerdict::Unavailable
    );
    let changed_environment = FakeResolver::with_responses([
        SourceResolution::Unavailable,
        SourceResolution::Unavailable,
    ]);
    assert_eq!(
        changed_environment
            .checked(|checked_resolver| Ok(evaluate_oracle(
                &original,
                &OracleContext::new(
                    &store,
                    &clock,
                    checked_resolver,
                    agent_memory::OracleEnvironment::new("linux", "x86_64")
                )
            )))?
            .verdict(),
        OracleVerdict::Unavailable
    );
    Ok(())
}

#[test]
fn declarative_oracle_digest_excludes_proof_while_proof_digest_binds_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let (root, store) = open_store(fixture.path())?;
    let clock = FixedClock::at("2026-08-28T00:00:00Z")?;
    for (id, locator, fingerprint) in [
        ('a', "https://docs.example.test/first", 'a'),
        ('b', "https://docs.example.test/second", 'b'),
    ] {
        let entry = entry(
            id,
            "invariant",
            &[SourceFixture {
                kind: "official-url",
                locator,
                fingerprint,
            }],
        )?;
        let resolver = FakeResolver::with_responses([valid(fingerprint)]);
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
    let entries = cache
        .get("entries")
        .ok_or("missing fixture element")?
        .as_array()
        .ok_or("missing fixture value")?;

    assert_eq!(
        entries
            .first()
            .ok_or("missing fixture element")?
            .get("oracle_digest")
            .ok_or("missing fixture element")?,
        entries
            .get(1)
            .ok_or("missing fixture element")?
            .get("oracle_digest")
            .ok_or("missing fixture element")?
    );
    assert_ne!(
        entries
            .first()
            .ok_or("missing fixture element")?
            .get("proof_digest")
            .ok_or("missing fixture element")?,
        entries
            .get(1)
            .ok_or("missing fixture element")?
            .get("proof_digest")
            .ok_or("missing fixture element")?
    );
    Ok(())
}
