use agent_memory::{
    HookAgent, InjectedMemory, MemoryKind, OmissionEffect, OmittedMemory, RetrievalReport,
    SourceKind, SourceSummary, render_hook_response,
};
use serde_json::{Value, json};
#[test]
fn renders_exact_supported_host_envelopes_with_redacted_context()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
    let _: () = for agent in [HookAgent::Codex, HookAgent::Claude] {
        let bytes = render_hook_response(agent, &report)?;
        let value: Value = serde_json::from_slice(&bytes)?;
        assert_eq!(
            value,
            json ! ({ "hookSpecificOutput" : { "hookEventName" : "UserPromptSubmit" , "additionalContext" : expected_context , } })
        );
        let context = (*value
            .pointer("/hookSpecificOutput/additionalContext")
            .ok_or("missing hook response /hookSpecificOutput/additionalContext")?)
        .as_str()
        .ok_or("expected JSON string")?;
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
    };
    Ok(())
}
#[test]
fn renders_no_context_for_an_empty_report() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let report = RetrievalReport {
        injected: Vec::new(),
        omitted: Vec::new(),
        omitted_by_limit: 0,
    };
    let _: () = for agent in [HookAgent::Codex, HookAgent::Claude] {
        assert_eq!(render_hook_response(agent, &report)?, b"{}");
    };
    Ok(())
}
#[test]
fn does_not_reconstruct_failure_classes_from_omission_codes()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let report = RetrievalReport {
        injected: vec![injected('a')],
        omitted: vec![omitted("oracle_unavailable")],
        omitted_by_limit: 0,
    };
    let _: () = for agent in [HookAgent::Codex, HookAgent::Claude] {
        let bytes = render_hook_response(agent, &report)?;
        let value: Value = serde_json::from_slice(&bytes)?;
        assert!(
            (*value
                .pointer("/hookSpecificOutput/additionalContext")
                .ok_or("missing hook response /hookSpecificOutput/additionalContext")?)
            .as_str()
            .ok_or("expected JSON string")?
            .contains("oracle_unavailable")
        );
    };
    Ok(())
}
#[test]
fn limits_defensive_rendering_to_five_injections()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let report = RetrievalReport {
        injected: ['a', 'b', 'c', 'd', 'e', 'f']
            .into_iter()
            .map(injected)
            .collect(),
        omitted: Vec::new(),
        omitted_by_limit: 0,
    };
    let bytes = render_hook_response(HookAgent::Codex, &report)?;
    let value: Value = serde_json::from_slice(&bytes)?;
    let context = (*value
        .pointer("/hookSpecificOutput/additionalContext")
        .ok_or("missing hook response /hookSpecificOutput/additionalContext")?)
    .as_str()
    .ok_or("expected JSON string")?;
    let payload: Value =
        serde_json::from_str(context.lines().nth(1).ok_or("context payload missing")?)?;
    assert_eq!(
        (*payload
            .pointer("/injected")
            .ok_or("missing hook response /injected")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len(),
        5
    );
    assert_eq!(
        (*payload
            .pointer("/omitted_counts/injection_limit")
            .ok_or("missing hook response /omitted_counts/injection_limit")?),
        1
    );
    assert!(!context.contains("Memory f"));
    Ok(())
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
pub fn injected(character: char) -> InjectedMemory {
    InjectedMemory {
        id: format!("mem_{}", character.to_string().repeat(24)),
        kind: MemoryKind::Invariant,
        statement: format!("Memory {character}"),
        sources: vec![SourceSummary::redacted(SourceKind::UserDecision)],
        verdict_age_milliseconds: 0,
    }
}
pub fn omitted(code: &str) -> OmittedMemory {
    OmittedMemory {
        id: "mem_bbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
        code: code.to_owned(),
        question: Some("raw-secret prompt?".to_owned()),
        effect: OmissionEffect::NotApplied,
    }
}
