use super::protocol::{injected, omitted};
use agent_memory::{
    HookAgent, InjectedMemory, MemoryKind, RetrievalReport, SourceKind, SourceSummary,
    render_hook_response,
};
use serde_json::Value;

#[test]
fn bounds_the_complete_utf8_context_and_reports_whole_omissions() {
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

    let first = render_hook_response(HookAgent::Codex, &report).unwrap();
    let second = render_hook_response(HookAgent::Codex, &report).unwrap();
    assert_eq!(first, second);
    let envelope: Value = serde_json::from_slice(&first).unwrap();
    let context = envelope["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(context.len() <= 2000);
    let payload: Value = serde_json::from_str(context.split_once('\n').unwrap().1).unwrap();
    assert_eq!(payload["injected"].as_array().unwrap().len(), 0);
    assert!(payload["omitted"].as_array().unwrap().len() <= 20);
    assert_eq!(payload["omitted_counts"]["retrieval_limit"], 7);
    assert_eq!(payload["omitted_counts"]["context_injections"], 5);
    assert_eq!(
        payload["omitted_counts"]["context_omissions"]
            .as_u64()
            .unwrap()
            + payload["omitted"].as_array().unwrap().len() as u64,
        100
    );
    for forbidden in ["locator", "index.json", "oracle-cache", ".local/share"] {
        assert!(!context.contains(forbidden));
    }
}

#[test]
fn skips_one_oversized_record_without_truncating_later_records() {
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

    let bytes = render_hook_response(HookAgent::Claude, &report).unwrap();
    let envelope: Value = serde_json::from_slice(&bytes).unwrap();
    let context = envelope["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    let payload: Value = serde_json::from_str(context.split_once('\n').unwrap().1).unwrap();
    assert_eq!(payload["injected"].as_array().unwrap().len(), 1);
    assert_eq!(payload["injected"][0]["statement"], "Memory b");
    assert_eq!(payload["omitted_counts"]["context_injections"], 1);
    assert!(!context.contains(&"é".repeat(10)));
}

#[test]
fn accepts_a_whole_record_at_the_exact_context_boundary() {
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
        let bytes = render_hook_response(HookAgent::Codex, &report).unwrap();
        let envelope: Value = serde_json::from_slice(&bytes).unwrap();
        let context = envelope["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap();
        let payload: Value = serde_json::from_str(context.split_once('\n').unwrap().1).unwrap();
        if context.len() == 2000 && payload["injected"].as_array().unwrap().len() == 1 {
            exact = Some(length);
        }
        if exact == length.checked_sub(1) && payload["injected"].as_array().unwrap().is_empty() {
            next_is_omitted = true;
            break;
        }
    }

    assert!(exact.is_some());
    assert!(next_is_omitted);
}
