use agent_memory::{
    HookAgent, InjectedMemory, MemoryKind, OmissionEffect, OmittedMemory, RetrievalReport,
    SourceKind, SourceSummary, render_hook_response,
};
use serde_json::{Value, json};

#[test]
fn renders_exact_supported_host_envelopes_with_redacted_context() {
    let report = report_with_one_injection();
    let expected_context = concat!(
        "AGENT_MEMORY_CONTEXT_V1\n",
        "{\"injected\":[{\"kind\":\"invariant\",\"statement\":\"Use durable architecture.\",",
        "\"sources\":[{\"kind\":\"git-file\",\"summary\":\"docs/adr/042.md\"},",
        "{\"kind\":\"official-url\",\"summary\":\"https://docs.example.test/memory\"},",
        "{\"kind\":\"local-file\",\"summary\":\"redacted\"}],",
        "\"verdict_age_milliseconds\":3600000}],",
        "\"omitted\":[{\"code\":\"oracle_needs_confirmation\",\"effect\":\"not_applied\"},",
        "{\"code\":\"selection_stale\",\"effect\":\"not_applied\"}],",
        "\"omitted_counts\":{\"retrieval_limit\":2,\"injection_limit\":0,",
        "\"context_injections\":0,\"context_omissions\":0}}"
    );

    for agent in [HookAgent::Codex, HookAgent::Claude] {
        let bytes = render_hook_response(agent, &report).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(
            value,
            json!({
                "hookSpecificOutput": {
                    "hookEventName": "UserPromptSubmit",
                    "additionalContext": expected_context,
                }
            })
        );
        let context = value["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap();
        for forbidden in [
            "mem_aaaaaaaa",
            "locator",
            "oracle-cache",
            "index.json",
            ".local/share",
            "raw-secret",
            "?private",
            "#fragment",
            "transcript",
            "prompt",
        ] {
            assert!(
                !context.contains(forbidden),
                "leaked {forbidden}: {context}"
            );
        }
    }
}

#[test]
fn renders_no_context_for_an_empty_report() {
    let report = RetrievalReport {
        injected: Vec::new(),
        omitted: Vec::new(),
        omitted_by_limit: 0,
    };

    for agent in [HookAgent::Codex, HookAgent::Claude] {
        assert_eq!(render_hook_response(agent, &report).unwrap(), b"{}");
    }
}

#[test]
fn does_not_reconstruct_failure_classes_from_omission_codes() {
    let report = RetrievalReport {
        injected: vec![injected('a')],
        omitted: vec![omitted("oracle_unavailable")],
        omitted_by_limit: 0,
    };

    for agent in [HookAgent::Codex, HookAgent::Claude] {
        let bytes = render_hook_response(agent, &report).unwrap();
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert!(
            value["hookSpecificOutput"]["additionalContext"]
                .as_str()
                .unwrap()
                .contains("oracle_unavailable")
        );
    }
}

#[test]
fn limits_defensive_rendering_to_five_injections() {
    let report = RetrievalReport {
        injected: ['a', 'b', 'c', 'd', 'e', 'f']
            .into_iter()
            .map(injected)
            .collect(),
        omitted: Vec::new(),
        omitted_by_limit: 0,
    };

    let bytes = render_hook_response(HookAgent::Codex, &report).unwrap();
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    let context = value["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    let payload: Value = serde_json::from_str(context.lines().nth(1).unwrap()).unwrap();
    assert_eq!(payload["injected"].as_array().unwrap().len(), 5);
    assert_eq!(payload["omitted_counts"]["injection_limit"], 1);
    assert!(!context.contains("Memory f"));
}

fn report_with_one_injection() -> RetrievalReport {
    RetrievalReport {
        injected: vec![InjectedMemory {
            id: "mem_aaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            kind: MemoryKind::Invariant,
            statement: "Use durable architecture.".to_owned(),
            sources: vec![
                SourceSummary::with_locator(SourceKind::GitFile, "docs/adr/042.md"),
                SourceSummary::with_locator(
                    SourceKind::OfficialUrl,
                    "https://docs.example.test/memory",
                ),
                SourceSummary::redacted(SourceKind::LocalFile),
            ],
            verdict_age_milliseconds: 3_600_000,
        }],
        omitted: vec![
            omitted("oracle_needs_confirmation"),
            omitted("selection_stale"),
        ],
        omitted_by_limit: 2,
    }
}

pub(super) fn injected(character: char) -> InjectedMemory {
    InjectedMemory {
        id: format!("mem_{}", character.to_string().repeat(24)),
        kind: MemoryKind::Invariant,
        statement: format!("Memory {character}"),
        sources: vec![SourceSummary::redacted(SourceKind::UserDecision)],
        verdict_age_milliseconds: 0,
    }
}

pub(super) fn omitted(code: &str) -> OmittedMemory {
    OmittedMemory {
        id: "mem_bbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
        code: code.to_owned(),
        question: Some("raw-secret prompt?".to_owned()),
        effect: OmissionEffect::NotApplied,
    }
}
