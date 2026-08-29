use super::support::*;
use agent_memory::{
    HumanConclusion, OmissionEffect, OracleEnvironment, RetrievalContext, RetrievalRequest,
    SourceResolution, Status, TransitionContext, TransitionVerdict, confirm, retrieve,
};
use std::fs;

type Conclusion = fn(&str) -> Result<HumanConclusion, agent_memory::MemoryError>;

#[test]
fn every_typed_human_business_terminal_transitions_once() {
    let cases: [(&str, char, Conclusion, Status); 5] = [
        (
            "goal",
            '1',
            HumanConclusion::goal_achieved,
            Status::Achieved,
        ),
        (
            "goal",
            '2',
            HumanConclusion::goal_abandoned,
            Status::Abandoned,
        ),
        (
            "decision",
            '3',
            HumanConclusion::decision_superseded,
            Status::Superseded,
        ),
        (
            "unknown",
            '4',
            HumanConclusion::unknown_resolved,
            Status::Resolved,
        ),
        (
            "assumption",
            '5',
            HumanConclusion::assumption_confirmed,
            Status::Confirmed,
        ),
    ];
    for (kind, id, conclusion, expected) in cases {
        let fixture = tempfile::tempdir().unwrap();
        let (root, store) = open_store(fixture.path());
        let yaml = entry_yaml(
            id,
            kind,
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:transition",
                fingerprint: 'a',
            }],
        );
        write_user_entry(&root, id, &yaml);
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        let id = format!("mem_{}", id.to_string().repeat(24));
        let result = confirm(
            &id,
            conclusion("Human conclusion established.").unwrap(),
            TransitionContext::new(&store, &clock),
        )
        .unwrap();

        assert_eq!(result.status(), expected, "{kind}");
        let stored = store.load(&id).unwrap().unwrap();
        let transition = stored.transition().unwrap();
        assert_eq!(transition.from(), Status::Active, "{kind}");
        assert_eq!(transition.to(), expected, "{kind}");
        assert_eq!(transition.verdict(), TransitionVerdict::Valid, "{kind}");
        assert_eq!(transition.reason(), "Human conclusion established.");
        assert_eq!(
            confirm(
                &id,
                conclusion("Repeated conclusion.").unwrap(),
                TransitionContext::new(&store, &clock)
            )
            .unwrap_err()
            .code(),
            "entry_not_active",
            "{kind}"
        );
    }
}

#[test]
fn refuses_empty_reasons_and_incompatible_human_terminals_without_mutation() {
    assert_eq!(
        HumanConclusion::goal_achieved("   ").unwrap_err().code(),
        "invalid_transition_reason"
    );
    for (kind, id) in [("evidence", '6'), ("invariant", '7'), ("decision", '8')] {
        let fixture = tempfile::tempdir().unwrap();
        let (root, store) = open_store(fixture.path());
        let yaml = entry_yaml(
            id,
            kind,
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:incompatible",
                fingerprint: 'b',
            }],
        );
        let path = write_user_entry(&root, id, &yaml);
        let before = fs::read(&path).unwrap();
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        let result = confirm(
            &format!("mem_{}", id.to_string().repeat(24)),
            HumanConclusion::goal_achieved("Wrong terminal.").unwrap(),
            TransitionContext::new(&store, &clock),
        );

        assert_eq!(result.unwrap_err().code(), "invalid_human_conclusion");
        assert_eq!(fs::read(path).unwrap(), before);
    }
}

#[test]
fn automated_invalidity_is_the_only_path_to_invalidated_for_every_kind() {
    for (index, kind) in [
        "goal",
        "decision",
        "evidence",
        "invariant",
        "unknown",
        "assumption",
    ]
    .into_iter()
    .enumerate()
    {
        let fixture = tempfile::tempdir().unwrap();
        let (root, store) = open_store(fixture.path());
        let id = char::from_digit((index + 9) as u32, 16).unwrap();
        let yaml = entry_yaml(
            id,
            kind,
            &[SourceFixture {
                kind: "local-file",
                locator: "/tmp/proof",
                fingerprint: 'a',
            }],
        );
        write_user_entry(&root, id, &yaml);
        let key = project_key(fixture.path());
        let selection = select(&store, &key, 5);
        let resolver = FakeResolver::with_responses([valid('b')]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        let report = retrieve(
            RetrievalRequest::new(&selection, &key, true),
            RetrievalContext::new(&store, &clock, &resolver, environment()),
        );

        assert!(report.injected.is_empty(), "{kind}");
        assert_eq!(report.omitted[0].code, "oracle_invalidated", "{kind}");
        assert_eq!(report.omitted[0].effect, OmissionEffect::NotApplied);
        let stored = store
            .load(&format!("mem_{}", id.to_string().repeat(24)))
            .unwrap()
            .unwrap();
        assert_eq!(stored.status(), Status::Invalidated, "{kind}");
        assert_eq!(
            stored.transition().unwrap().verdict(),
            TransitionVerdict::Invalid,
            "{kind}"
        );
    }
}

#[test]
fn unavailable_and_needs_confirmation_leave_yaml_byte_identical() {
    for (id, source, response, expected) in [
        (
            'a',
            SourceFixture {
                kind: "official-url",
                locator: "https://docs.example.test/unavailable",
                fingerprint: 'a',
            },
            Some(SourceResolution::Unavailable),
            "oracle_unavailable",
        ),
        (
            'b',
            SourceFixture {
                kind: "user-decision",
                locator: "decision:confirmation",
                fingerprint: 'b',
            },
            None,
            "oracle_needs_confirmation",
        ),
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let (root, store) = open_store(fixture.path());
        let yaml = entry_yaml(id, "invariant", &[source]);
        let path = write_user_entry(&root, id, &yaml);
        let before = fs::read(&path).unwrap();
        let key = project_key(fixture.path());
        let selection = select(&store, &key, 5);
        let resolver = FakeResolver::with_responses(response);
        let clock = FixedClock::at("2026-08-28T01:00:00Z");
        let report = retrieve(
            RetrievalRequest::new(&selection, &key, true),
            RetrievalContext::new(
                &store,
                &clock,
                &resolver,
                OracleEnvironment::new("macos", "aarch64"),
            ),
        );

        assert_eq!(report.omitted[0].code, expected);
        assert_eq!(report.omitted[0].effect, OmissionEffect::NotApplied);
        assert!(report.omitted[0].question.is_some());
        assert_eq!(fs::read(path).unwrap(), before);
    }
}
