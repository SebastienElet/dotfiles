use super::{assert_usage_error, claude_usage, codex_usage};
use agent_handoff::{Agent, Usage, find_latest_usage};

const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;

#[test]
fn integral_json_number_representations_match_javascript_safe_integers() {
    let cases = [("1.0", 1), ("1e3", 1_000), ("-0", 0)];

    for (input_tokens, expected) in cases {
        assert_eq!(
            find_latest_usage(&claude_usage(&format!(r#""input_tokens":{input_tokens}"#)))
                .unwrap()
                .used,
            expected
        );
    }
}

#[test]
fn out_of_range_json_numbers_keep_field_diagnostics() {
    let cases = [
        (
            claude_usage(r#""input_tokens":1e400"#),
            "invalid Claude input_tokens",
        ),
        (
            claude_usage(r#""input_tokens":0,"cache_read_input_tokens":1e400"#),
            "invalid Claude cache_read_input_tokens",
        ),
        (
            claude_usage(r#""input_tokens":0,"cache_creation_input_tokens":1e400"#),
            "invalid Claude cache_creation_input_tokens",
        ),
        (
            codex_usage("1e400", "100", "0"),
            "invalid Codex input_tokens",
        ),
        (
            codex_usage("1", "1e400", "0"),
            "invalid Codex model_context_window",
        ),
    ];

    for (transcript, message) in cases {
        assert_usage_error(&transcript, message);
    }
}

#[test]
fn claude_numeric_fields_enforce_javascript_safe_integer_bounds() {
    assert_eq!(
        find_latest_usage(&claude_usage(&format!(
            r#""input_tokens":{MAX_SAFE_INTEGER}"#
        )))
        .unwrap()
        .used,
        MAX_SAFE_INTEGER
    );

    let invalid_fields = [
        (r#""input_tokens":-1"#.into(), "invalid Claude input_tokens"),
        (
            r#""input_tokens":1.5"#.into(),
            "invalid Claude input_tokens",
        ),
        (
            format!(r#""input_tokens":{}"#, MAX_SAFE_INTEGER + 1),
            "invalid Claude input_tokens",
        ),
        (
            r#""input_tokens":0,"cache_read_input_tokens":-1"#.into(),
            "invalid Claude cache_read_input_tokens",
        ),
        (
            r#""input_tokens":0,"cache_read_input_tokens":1.5"#.into(),
            "invalid Claude cache_read_input_tokens",
        ),
        (
            format!(
                r#""input_tokens":0,"cache_read_input_tokens":{}"#,
                MAX_SAFE_INTEGER + 1
            ),
            "invalid Claude cache_read_input_tokens",
        ),
        (
            r#""input_tokens":0,"cache_creation_input_tokens":-1"#.into(),
            "invalid Claude cache_creation_input_tokens",
        ),
        (
            r#""input_tokens":0,"cache_creation_input_tokens":1.5"#.into(),
            "invalid Claude cache_creation_input_tokens",
        ),
        (
            format!(
                r#""input_tokens":0,"cache_creation_input_tokens":{}"#,
                MAX_SAFE_INTEGER + 1
            ),
            "invalid Claude cache_creation_input_tokens",
        ),
        (
            format!(r#""input_tokens":{MAX_SAFE_INTEGER},"cache_read_input_tokens":1"#),
            "invalid Claude token total",
        ),
    ];

    for (fields, message) in invalid_fields {
        assert_usage_error(&claude_usage(&fields), message);
    }
}

#[test]
fn codex_numeric_fields_enforce_javascript_safe_integer_bounds() {
    assert_eq!(
        find_latest_usage(&codex_usage(
            &MAX_SAFE_INTEGER.to_string(),
            &MAX_SAFE_INTEGER.to_string(),
            "0",
        ))
        .unwrap(),
        Usage {
            agent: Agent::Codex,
            used: MAX_SAFE_INTEGER,
            window: Some(MAX_SAFE_INTEGER),
        }
    );

    let invalid_values = [
        ("-1".into(), "100".into(), "invalid Codex input_tokens"),
        ("1.5".into(), "100".into(), "invalid Codex input_tokens"),
        (
            (MAX_SAFE_INTEGER + 1).to_string(),
            "100".into(),
            "invalid Codex input_tokens",
        ),
        ("1".into(), "0".into(), "invalid Codex model_context_window"),
        (
            "1".into(),
            "-1".into(),
            "invalid Codex model_context_window",
        ),
        (
            "1".into(),
            "1.5".into(),
            "invalid Codex model_context_window",
        ),
        (
            "1".into(),
            (MAX_SAFE_INTEGER + 1).to_string(),
            "invalid Codex model_context_window",
        ),
    ];

    for (input_tokens, window, message) in invalid_values {
        assert_usage_error(&codex_usage(&input_tokens, &window, "0"), message);
    }
}
