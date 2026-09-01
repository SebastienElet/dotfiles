use super::support::*;
use agent_memory::{OracleContext, OracleVerdict, SourceResolution, evaluate_oracle};

#[test]
fn persists_only_valid_verdicts_with_canonical_ordered_cache_fields() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
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
    );
    let resolver = FakeResolver::with_responses([valid('a'), valid('b')]);
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    evaluate_oracle(
        &valid_entry,
        OracleContext::new(&store, &clock, &resolver, environment()),
    );

    let cache = cache_json(&root);
    assert_eq!(cache["schema_version"], 1);
    let record = &cache["entries"][0];
    assert_eq!(record["entry_id"], user_entry_id('5', "invariant"));
    assert_eq!(record["verdict"], "valid");
    assert_eq!(record["validated_at"], "2026-08-28T00:00:00Z");
    assert_eq!(record["environment"]["os"], "macos");
    assert_eq!(record["environment"]["arch"], "aarch64");
    assert_eq!(record["source_fingerprints"][0]["kind"], "official-url");
    assert_eq!(
        record["source_fingerprints"][0]["fingerprint"],
        fingerprint('a')
    );
    assert_eq!(record["source_fingerprints"][1]["kind"], "local-file");
    assert_eq!(
        record["source_fingerprints"][1]["fingerprint"],
        fingerprint('b')
    );
    assert_eq!(
        record["oracle_digest"],
        "sha256:92c5240dc81207720563324941353993080e84180b689efc52af46840add96c9"
    );
    assert_eq!(
        record["proof_digest"],
        "sha256:625882fbac49144bea8bd2b2d9079b287544e53a02cab481bc49a4d199bff679"
    );
    let serialized = serde_json::to_string(&cache).unwrap();
    assert!(!serialized.contains("https://docs.example.test/first"));
    assert!(!serialized.contains("/tmp/second"));
}

#[test]
fn never_caches_invalid_unavailable_or_needs_confirmation() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
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
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    let invalid = FakeResolver::with_responses([valid('b')]);
    assert_eq!(
        evaluate_oracle(
            &automated('6'),
            OracleContext::new(&store, &clock, &invalid, environment())
        )
        .verdict(),
        OracleVerdict::Invalid
    );
    let unavailable = FakeResolver::with_responses([SourceResolution::Unavailable]);
    assert_eq!(
        evaluate_oracle(
            &automated('7'),
            OracleContext::new(&store, &clock, &unavailable, environment())
        )
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
    );
    let resolver = FakeResolver::with_responses([]);
    assert_eq!(
        evaluate_oracle(
            &decision,
            OracleContext::new(&store, &clock, &resolver, environment())
        )
        .verdict(),
        OracleVerdict::NeedsConfirmation
    );

    assert!(cache_json(&root)["entries"].as_array().unwrap().is_empty());
}

#[test]
fn environment_and_source_order_are_part_of_cache_identity() {
    let fixture = tempfile::tempdir().unwrap();
    let (_, store) = open_store(fixture.path());
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
    );
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
    let initial = FakeResolver::with_responses([valid('a'), valid('b')]);
    evaluate_oracle(
        &original,
        OracleContext::new(&store, &clock, &initial, environment()),
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
    );
    let changed_order = FakeResolver::with_responses([
        SourceResolution::Unavailable,
        SourceResolution::Unavailable,
    ]);
    assert_eq!(
        evaluate_oracle(
            &reversed,
            OracleContext::new(&store, &clock, &changed_order, environment())
        )
        .verdict(),
        OracleVerdict::Unavailable
    );
    let changed_environment = FakeResolver::with_responses([
        SourceResolution::Unavailable,
        SourceResolution::Unavailable,
    ]);
    assert_eq!(
        evaluate_oracle(
            &original,
            OracleContext::new(
                &store,
                &clock,
                &changed_environment,
                agent_memory::OracleEnvironment::new("linux", "x86_64")
            )
        )
        .verdict(),
        OracleVerdict::Unavailable
    );
}

#[test]
fn declarative_oracle_digest_excludes_proof_while_proof_digest_binds_it() {
    let fixture = tempfile::tempdir().unwrap();
    let (root, store) = open_store(fixture.path());
    let clock = FixedClock::at("2026-08-28T00:00:00Z");
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
        );
        let resolver = FakeResolver::with_responses([valid(fingerprint)]);
        evaluate_oracle(
            &entry,
            OracleContext::new(&store, &clock, &resolver, environment()),
        );
    }
    let cache = cache_json(&root);
    let entries = cache["entries"].as_array().unwrap();

    assert_eq!(entries[0]["oracle_digest"], entries[1]["oracle_digest"]);
    assert_ne!(entries[0]["proof_digest"], entries[1]["proof_digest"]);
}
