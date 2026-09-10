#![cfg(test)]

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

use agent_handoff::{Agent, Environment, HandoffError, Usage, handoff_output, select_threshold};
use std::ffi::{OsStr, OsString};

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
    assert_eq!(
        environment.xdg_state_home.as_deref(),
        Some(OsStr::new("/state"))
    );
    assert_eq!(environment.home.as_deref(), Some(OsStr::new("/home")));
}

#[test]
fn explicit_threshold_takes_priority_over_context_windows() -> TestResult {
    assert_eq!(
        select_threshold(
            &codex(0, 100_001),
            &environment(&[
                ("HANDOFF_TOKEN_THRESHOLD", "50000"),
                ("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "not-a-number"),
            ]),
        )?,
        50_000
    );
    Ok(())
}

#[test]
fn claude_window_fallback_uses_an_exact_integer_floor() -> TestResult {
    assert_eq!(
        select_threshold(
            &claude(0),
            &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "100000")]),
        )?,
        85_000
    );
    assert_eq!(
        select_threshold(
            &claude(0),
            &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "9007199254740991",)]),
        )?,
        7_656_119_366_529_842
    );
    Ok(())
}

#[test]
fn codex_window_takes_priority_over_claude_environment_window() -> TestResult {
    assert_eq!(
        select_threshold(
            &codex(0, 100_001),
            &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "not-a-number")]),
        )?,
        85_000
    );
    Ok(())
}

#[test]
fn direct_usage_rejects_invalid_codex_windows() -> TestResult {
    for window in [0, MAX_SAFE_INTEGER + 1, u64::MAX] {
        assert_eq!(
            select_threshold(&codex(0, window), &Environment::default())
                .err()
                .ok_or("expected operation to fail")?,
            HandoffError::usage("invalid Codex model_context_window")
        );
    }
    Ok(())
}

#[test]
fn empty_explicit_threshold_falls_back_to_the_context_window() -> TestResult {
    assert_eq!(
        select_threshold(
            &claude(0),
            &environment(&[
                ("HANDOFF_TOKEN_THRESHOLD", ""),
                ("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "100000"),
            ]),
        )?,
        85_000
    );
    Ok(())
}

#[test]
fn absent_or_empty_claude_window_is_rejected() -> TestResult {
    for environment in [
        Environment::default(),
        environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", "")]),
    ] {
        assert_eq!(
            select_threshold(&claude(0), &environment)
                .err()
                .ok_or("expected operation to fail")?,
            HandoffError::usage("missing context window")
        );
    }
    Ok(())
}

#[test]
fn invalid_positive_integer_forms_are_rejected() -> TestResult {
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
            .err()
            .ok_or("expected operation to fail")?,
            HandoffError::usage("invalid HANDOFF_TOKEN_THRESHOLD")
        );
        assert_eq!(
            select_threshold(
                &claude(0),
                &environment(&[("CLAUDE_CODE_AUTO_COMPACT_WINDOW", value)]),
            )
            .err()
            .ok_or("expected operation to fail")?,
            HandoffError::usage("invalid CLAUDE_CODE_AUTO_COMPACT_WINDOW")
        );
    }
    Ok(())
}

#[test]
fn handoff_output_matches_claude_hook_bytes() -> TestResult {
    assert_eq!(
        handoff_output(&claude(85_000), 85_000)?,
        b"{\n  \"decision\": \"block\",\n  \"reason\": \"Context is at 85k tokens, past the 85k handoff threshold. Start no new work. Use /handoff to emit the resume prompt for a fresh session, then stop.\"\n}\n",
    );
    Ok(())
}

#[test]
fn handoff_output_uses_codex_invocation_and_token_floors() -> TestResult {
    assert_eq!(
        handoff_output(&codex(85_999, MAX_SAFE_INTEGER), 50_999)?,
        b"{\n  \"decision\": \"block\",\n  \"reason\": \"Context is at 85k tokens, past the 50k handoff threshold. Start no new work. Use $handoff to emit the resume prompt for a fresh session, then stop.\"\n}\n",
    );
    Ok(())
}
