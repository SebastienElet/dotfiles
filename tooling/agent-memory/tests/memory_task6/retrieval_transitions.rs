use super::support::*;
use agent_memory::{
    HumanConclusion, OmissionEffect, OracleEnvironment, RetrievalContext, RetrievalRequest,
    SourceResolution, Status, TransitionContext, TransitionVerdict, confirm, retrieve,
};
use std::fs;

type Conclusion = fn(&str) -> Result<HumanConclusion, agent_memory::MemoryError>;

#[test]
fn every_typed_human_business_terminal_transitions_once()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        let fixture = tempfile::tempdir()?;
        let (root, store) = open_store(fixture.path())?;
        let yaml = entry_yaml(
            id,
            kind,
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:transition",
                fingerprint: 'a',
            }],
        )?;
        write_user_entry(&root, id, &yaml)?;
        let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
        let id = user_entry_id(id, kind);
        let result = confirm(
            &id,
            conclusion("Human conclusion established.")?,
            TransitionContext::new(&store, &clock),
        )?;

        assert_eq!(result.status(), expected, "{kind}");
        let stored = store.load(&id)?.ok_or("missing fixture value")?;
        let transition = stored.transition().ok_or("missing fixture value")?;
        assert_eq!(transition.from(), Status::Active, "{kind}");
        assert_eq!(transition.to(), expected, "{kind}");
        assert_eq!(transition.verdict(), TransitionVerdict::Valid, "{kind}");
        assert_eq!(transition.reason(), "Human conclusion established.");
        assert_eq!(
            confirm(
                &id,
                conclusion("Repeated conclusion.")?,
                TransitionContext::new(&store, &clock)
            )
            .err()
            .ok_or("expected operation failure")?
            .code(),
            "entry_not_active",
            "{kind}"
        );
    }
    Ok(())
}

#[test]
fn refuses_empty_reasons_and_incompatible_human_terminals_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_eq!(
        HumanConclusion::goal_achieved("   ")
            .err()
            .ok_or("expected operation failure")?
            .code(),
        "invalid_transition_reason"
    );
    for (kind, id) in [("evidence", '6'), ("invariant", '7'), ("decision", '8')] {
        let fixture = tempfile::tempdir()?;
        let (root, store) = open_store(fixture.path())?;
        let yaml = entry_yaml(
            id,
            kind,
            &[SourceFixture {
                kind: "user-decision",
                locator: "decision:incompatible",
                fingerprint: 'b',
            }],
        )?;
        let path = write_user_entry(&root, id, &yaml)?;
        let before = fs::read(&path)?;
        let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
        let result = confirm(
            &user_entry_id(id, kind),
            HumanConclusion::goal_achieved("Wrong terminal.")?,
            TransitionContext::new(&store, &clock),
        );

        assert_eq!(
            result.err().ok_or("expected operation failure")?.code(),
            "invalid_human_conclusion"
        );
        assert_eq!(fs::read(path)?, before);
    }
    Ok(())
}

#[test]
fn automated_invalidity_is_the_only_path_to_invalidated_for_every_kind()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        let fixture = tempfile::tempdir()?;
        let (root, store) = open_store(fixture.path())?;
        let id = char::from_digit(u32::try_from(index + 9)?, 16).ok_or("missing fixture value")?;
        let yaml = entry_yaml(
            id,
            kind,
            &[SourceFixture {
                kind: "local-file",
                locator: "/tmp/proof",
                fingerprint: 'a',
            }],
        )?;
        write_user_entry(&root, id, &yaml)?;
        let key = project_key(fixture.path())?;
        let selection = select(&store, &key, 5)?;
        let resolver = FakeResolver::with_responses([valid('b')]);
        let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
        let report = resolver.checked(|checked_resolver| {
            Ok(retrieve(
                RetrievalRequest::new(&selection, &key, true),
                &RetrievalContext::new(&store, &clock, checked_resolver, environment()),
            ))
        })?;

        assert!(report.injected.is_empty(), "{kind}");
        assert_eq!(
            report
                .omitted
                .first()
                .ok_or("missing fixture element")?
                .code,
            "oracle_invalidated",
            "{kind}"
        );
        assert_eq!(
            report
                .omitted
                .first()
                .ok_or("missing fixture element")?
                .effect,
            OmissionEffect::NotApplied
        );
        let stored = store
            .load(&user_entry_id(id, kind))?
            .ok_or("missing fixture value")?;
        assert_eq!(stored.status(), Status::Invalidated, "{kind}");
        assert_eq!(
            stored
                .transition()
                .ok_or("missing fixture value")?
                .verdict(),
            TransitionVerdict::Invalid,
            "{kind}"
        );
    }
    Ok(())
}

#[test]
fn unavailable_and_needs_confirmation_leave_yaml_byte_identical()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        let fixture = tempfile::tempdir()?;
        let (root, store) = open_store(fixture.path())?;
        let yaml = entry_yaml(id, "invariant", &[source])?;
        let path = write_user_entry(&root, id, &yaml)?;
        let before = fs::read(&path)?;
        let key = project_key(fixture.path())?;
        let selection = select(&store, &key, 5)?;
        let resolver = FakeResolver::with_responses(response);
        let clock = FixedClock::at("2026-08-28T01:00:00Z")?;
        let report = resolver.checked(|checked_resolver| {
            Ok(retrieve(
                RetrievalRequest::new(&selection, &key, true),
                &RetrievalContext::new(
                    &store,
                    &clock,
                    checked_resolver,
                    OracleEnvironment::new("macos", "aarch64"),
                ),
            ))
        })?;

        assert_eq!(
            report
                .omitted
                .first()
                .ok_or("missing fixture element")?
                .code,
            expected
        );
        assert_eq!(
            report
                .omitted
                .first()
                .ok_or("missing fixture element")?
                .effect,
            OmissionEffect::NotApplied
        );
        assert!(
            report
                .omitted
                .first()
                .ok_or("missing fixture element")?
                .question
                .is_some()
        );
        assert_eq!(fs::read(path)?, before);
    }
    Ok(())
}
