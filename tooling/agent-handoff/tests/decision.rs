use agent_handoff::{Agent, Environment, HandoffError, Usage, handoff_output, select_threshold};
use std::ffi::OsString;

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

fn environment(values: &[(&str, &str)]) -> Environment {
    Environment::from_iter(
        values
            .iter()
            .map(|(name, value)| (OsString::from(name), OsString::from(value))),
    )
}

fn claude(used: u64) -> Usage {
    Usage {
        agent: Agent::ClaudeCode,
        used,
        window: None,
    }
}

fn codex(used: u64, window: u64) -> Usage {
    Usage {
        agent: Agent::Codex,
        used,
        window: Some(window),
    }
}

#[test]
fn environment_retains_only_handoff_contract_variables() {
    let environment = Environment::from_iter([
        (
            OsString::from("HANDOFF_TOKEN_THRESHOLD"),
            OsString::from("50000"),
        ),
        (
            OsString::from("CLAUDE_CODE_AUTO_COMPACT_WINDOW"),
            OsString::from("100000"),
        ),
        (OsString::from("XDG_STATE_HOME"), OsString::from("/state")),
        (OsString::from("HOME"), OsString::from("/home")),
        (OsString::from("UNRELATED"), OsString::from("discarded")),
    ]);

    assert_eq!(
        environment.handoff_token_threshold.as_deref(),
        Some("50000")
    );
    assert_eq!(
        environment.claude_code_auto_compact_window.as_deref(),
        Some("100000")
    );
    assert_eq!(environment.xdg_state_home.as_deref(), Some("/state"));
    assert_eq!(environment.home.as_deref(), Some("/home"));
}

#[test]
fn explicit_threshold_takes_priority_over_context_windows() {
    assert_eq!(
        select_threshold(
            &codex(0, 100_001),
            &environment(&[
                ("HANDOFF_TOKEN_THRESHOLD", "50000"),
                ("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "not-a-number"),
            ]),
        )
        .unwrap(),
        50_000
    );
}

#[test]
fn claude_window_fallback_uses_an_exact_integer_floor() {
    assert_eq!(
        select_threshold(
            &claude(0),
            &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "100000")]),
        )
        .unwrap(),
        85_000
    );
    assert_eq!(
        select_threshold(
            &claude(0),
            &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "9007199254740991",)]),
        )
        .unwrap(),
        7_656_119_366_529_842
    );
}

#[test]
fn codex_window_takes_priority_over_claude_environment_window() {
    assert_eq!(
        select_threshold(
            &codex(0, 100_001),
            &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "not-a-number")]),
        )
        .unwrap(),
        85_000
    );
}

#[test]
fn empty_explicit_threshold_falls_back_to_the_context_window() {
    assert_eq!(
        select_threshold(
            &claude(0),
            &environment(&[
                ("HANDOFF_TOKEN_THRESHOLD", ""),
                ("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "100000"),
            ]),
        )
        .unwrap(),
        85_000
    );
}

#[test]
fn absent_or_empty_claude_window_is_rejected() {
    for environment in [
        Environment::default(),
        environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "")]),
    ] {
        assert_eq!(
            select_threshold(&claude(0), &environment).unwrap_err(),
            HandoffError::usage("missing context window")
        );
    }
}

#[test]
fn invalid_positive_integer_forms_are_rejected() {
    let invalid_values = [
        "0",
        "01",
        "-1",
        "+1",
        " 1",
        "1 ",
        "1.5",
        "1e3",
        "85k",
        "９",
        "9007199254740992",
    ];

    for value in invalid_values {
        assert_eq!(
            select_threshold(
                &claude(0),
                &environment(&[("HANDOFF_TOKEN_THRESHOLD", value)]),
            )
            .unwrap_err(),
            HandoffError::usage("invalid HANDOFF_TOKEN_THRESHOLD")
        );
        assert_eq!(
            select_threshold(
                &claude(0),
                &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", value)]),
            )
            .unwrap_err(),
            HandoffError::usage("invalid CLAUDE_CODE_AUTO_COMPACT_WINDOW")
        );
    }
}

#[test]
fn handoff_output_matches_claude_hook_bytes() {
    assert_eq!(
        handoff_output(&claude(85_000), 85_000),
        b"{\n  \"decision\": \"block\",\n  \"reason\": \"Context is at 85k tokens, past the 85k handoff threshold. Start no new work. Use /handoff to emit the resume prompt for a fresh session, then stop.\"\n}\n",
    );
}

#[test]
fn handoff_output_uses_codex_invocation_and_token_floors() {
    assert_eq!(
        handoff_output(&codex(85_999, MAX_SAFE_INTEGER), 50_999),
        b"{\n  \"decision\": \"block\",\n  \"reason\": \"Context is at 85k tokens, past the 50k handoff threshold. Start no new work. Use $handoff to emit the resume prompt for a fresh session, then stop.\"\n}\n",
    );
}
