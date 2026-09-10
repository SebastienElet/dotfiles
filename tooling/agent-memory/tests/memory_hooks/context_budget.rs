use super::protocol::{injected, omitted};
use agent_memory::{
    HookAgent, InjectedMemory, MemoryKind, RetrievalReport, SourceKind, SourceSummary,
    render_hook_response,
};
use serde_json::Value;
#[test]
fn bounds_the_complete_utf8_context_and_reports_whole_omissions()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sources = (0..20)
        .map(|index| {
            SourceSummary::with_locator(
                SourceKind::GitFile,
                format!("docs/{index}-{}-é.md", "x".repeat(160)),
            )
        })
        .collect::<Vec<_>>();
    let report = RetrievalReport {
        injected: (0..5)
            .map(|index| InjectedMemory {
                id: format!("mem_{index:024}"),
                kind: MemoryKind::Invariant,
                statement: format!("é\\\"{}", "x".repeat(300)),
                sources: sources.clone(),
                verdict_age_milliseconds: index,
            })
            .collect(),
        omitted: (0..100)
            .map(|index| omitted(&format!("diagnostic_{index:03}_{}", "y".repeat(80))))
            .collect(),
        omitted_by_limit: 7,
    };
    let first = render_hook_response(HookAgent::Codex, &report)?;
    let second = render_hook_response(HookAgent::Codex, &report)?;
    assert_eq!(first, second);
    let envelope: Value = serde_json::from_slice(&first)?;
    let context = (*envelope
        .pointer("/hookSpecificOutput/additionalContext")
        .ok_or("missing hook response /hookSpecificOutput/additionalContext")?)
    .as_str()
    .ok_or("expected JSON string")?;
    assert!(context.len() <= 2000);
    let payload: Value = serde_json::from_str(
        context
            .split_once('\n')
            .ok_or("context header delimiter missing")?
            .1,
    )?;
    assert_eq!(
        (*payload
            .pointer("/injected")
            .ok_or("missing hook response /injected")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len(),
        0
    );
    assert!(
        (*payload
            .pointer("/omitted")
            .ok_or("missing hook response /omitted")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len()
            <= 20
    );
    assert_eq!(
        (*payload
            .pointer("/omitted_counts/retrieval_limit")
            .ok_or("missing hook response /omitted_counts/retrieval_limit")?),
        7
    );
    assert_eq!(
        (*payload
            .pointer("/omitted_counts/context_injections")
            .ok_or("missing hook response /omitted_counts/context_injections")?),
        5
    );
    assert_eq!(
        (*payload
            .pointer("/omitted_counts/context_omissions")
            .ok_or("missing hook response /omitted_counts/context_omissions")?)
        .as_u64()
        .ok_or("expected unsigned JSON integer")?
            + (*payload
                .pointer("/omitted")
                .ok_or("missing hook response /omitted")?)
            .as_array()
            .ok_or("expected JSON array")?
            .len() as u64,
        100
    );
    let _: () = for forbidden in ["locator", "index.json", "oracle-cache", ".local/share"] {
        assert!(!context.contains(forbidden));
    };
    Ok(())
}
#[test]
fn skips_one_oversized_record_without_truncating_later_records()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let report = RetrievalReport {
        injected: vec![
            InjectedMemory {
                statement: "é".repeat(2000),
                ..injected('a')
            },
            injected('b'),
        ],
        omitted: Vec::new(),
        omitted_by_limit: 0,
    };
    let bytes = render_hook_response(HookAgent::Claude, &report)?;
    let envelope: Value = serde_json::from_slice(&bytes)?;
    let context = (*envelope
        .pointer("/hookSpecificOutput/additionalContext")
        .ok_or("missing hook response /hookSpecificOutput/additionalContext")?)
    .as_str()
    .ok_or("expected JSON string")?;
    let payload: Value = serde_json::from_str(
        context
            .split_once('\n')
            .ok_or("context header delimiter missing")?
            .1,
    )?;
    assert_eq!(
        (*payload
            .pointer("/injected")
            .ok_or("missing hook response /injected")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len(),
        1
    );
    assert_eq!(
        (*payload
            .pointer("/injected/0/statement")
            .ok_or("missing hook response /injected/0/statement")?),
        "Memory b"
    );
    assert_eq!(
        (*payload
            .pointer("/omitted_counts/context_injections")
            .ok_or("missing hook response /omitted_counts/context_injections")?),
        1
    );
    assert!(!context.contains(&"é".repeat(10)));
    Ok(())
}
#[test]
fn accepts_a_whole_record_at_the_exact_context_boundary()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut exact = None;
    let mut next_is_omitted = false;
    for length in 0..2200 {
        let report = RetrievalReport {
            injected: vec![InjectedMemory {
                statement: "x".repeat(length),
                ..injected('a')
            }],
            omitted: Vec::new(),
            omitted_by_limit: 0,
        };
        let bytes = render_hook_response(HookAgent::Codex, &report)?;
        let envelope: Value = serde_json::from_slice(&bytes)?;
        let context = (*envelope
            .pointer("/hookSpecificOutput/additionalContext")
            .ok_or("missing hook response /hookSpecificOutput/additionalContext")?)
        .as_str()
        .ok_or("expected JSON string")?;
        let payload: Value = serde_json::from_str(
            context
                .split_once('\n')
                .ok_or("context header delimiter missing")?
                .1,
        )?;
        if context.len() == 2000
            && (*payload
                .pointer("/injected")
                .ok_or("missing hook response /injected")?)
            .as_array()
            .ok_or("expected JSON array")?
            .len()
                == 1
        {
            exact = Some(length);
        }
        if exact == length.checked_sub(1)
            && (*payload
                .pointer("/injected")
                .ok_or("missing hook response /injected")?)
            .as_array()
            .ok_or("expected JSON array")?
            .is_empty()
        {
            next_is_omitted = true;
            break;
        }
    }
    assert!(exact.is_some());
    assert!(next_is_omitted);
    Ok(())
}
