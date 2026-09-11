#![cfg(test)]

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error + Send + Sync>>;

use agent_handoff::{Agent, HandoffError, Usage, find_latest_usage};

#[path = "transcript/numeric.rs"]
mod numeric;

fn claude_usage(usage_fields: &str) -> String {
    format!(
        r#"{{"type":"assistant","isSidechain":false,"message":{{"usage":{{{usage_fields}}}}}}}"#
    )
}

fn claude_sidechain(sidechain: &str, usage_fields: &str) -> String {
    format!(
        r#"{{"type":"assistant","isSidechain":{sidechain},"message":{{"usage":{{{usage_fields}}}}}}}"#
    )
}

fn codex_usage(input_tokens: &str, window: &str, total_input_tokens: &str) -> String {
    r#"{"type":"event_msg","payload":{"type":"token_count","info":{"last_token_usage":{"input_tokens":INPUT},"model_context_window":WINDOW,"total_token_usage":{"input_tokens":TOTAL}}}}"#
        .replace("INPUT", input_tokens)
        .replace("WINDOW", window)
        .replace("TOTAL", total_input_tokens)
}

fn assert_usage_error(transcript: &str, message: &str) {
    assert_eq!(
        find_latest_usage(transcript),
        Err(HandoffError::usage(message))
    );
}

fn latest_usage(transcript: &str) -> TestResult<Usage> {
    Ok(find_latest_usage(transcript)?.ok_or("expected a supported usage record")?)
}

#[test]
fn claude_usage_sums_input_and_optional_cache_tokens() -> TestResult {
    assert_eq!(
        latest_usage(&claude_usage(r#""input_tokens":84999"#))?,
        Usage {
            agent: Agent::ClaudeCode,
            used: 84_999,
            window: None,
        }
    );
    assert_eq!(
        latest_usage(&claude_usage(
            r#""input_tokens":40000,"cache_read_input_tokens":20000,"cache_creation_input_tokens":30000"#
        ))
        ?
        .used,
        90_000
    );
    Ok(())
}

#[test]
fn codex_usage_reads_only_last_usage_and_its_context_window() -> TestResult {
    assert_eq!(
        latest_usage(&codex_usage("90000", "100000", "1"))?,
        Usage {
            agent: Agent::Codex,
            used: 90_000,
            window: Some(100_000),
        }
    );
    assert_eq!(
        latest_usage(&codex_usage("7", "8", "9007199254740992"))?.used,
        7
    );
    Ok(())
}

#[test]
fn latest_supported_main_chain_record_wins() -> TestResult {
    let transcript = [
        claude_usage(r#""input_tokens":10"#),
        claude_sidechain("true", r#""input_tokens":999"#),
        codex_usage("20", "100", "20"),
        r#"{"type":"other"}"#.into(),
    ]
    .join("\n");
    assert_eq!(latest_usage(&transcript)?.agent, Agent::Codex);
    assert_eq!(latest_usage(&transcript)?.used, 20);

    let later_claude = format!(
        "{transcript}\n{}",
        claude_sidechain("false", r#""input_tokens":30"#)
    );
    assert_eq!(
        latest_usage(&later_claude)?,
        Usage {
            agent: Agent::ClaudeCode,
            used: 30,
            window: None,
        }
    );
    Ok(())
}

#[test]
fn sidechains_are_ignored_and_invalid_markers_fail_closed() -> TestResult {
    assert_eq!(
        find_latest_usage(&claude_sidechain("true", r#""input_tokens":90000"#))?,
        None
    );
    assert_usage_error(
        &claude_sidechain(r#""true""#, r#""input_tokens":90000"#),
        "invalid Claude isSidechain",
    );
    Ok(())
}

#[test]
fn only_the_latest_five_hundred_physical_lines_are_retained() -> TestResult {
    let usage = claude_usage(r#""input_tokens":90000"#);
    let four_hundred_ninety_nine_lines = format!("{usage}{}", "\n".repeat(499));
    assert_eq!(latest_usage(&four_hundred_ninety_nine_lines)?.used, 90_000);

    let five_hundred_lines = format!("{usage}{}", "\n".repeat(500));
    assert_eq!(latest_usage(&five_hundred_lines)?.used, 90_000);

    let five_hundred_one_lines = format!("{usage}{}", "\n".repeat(501));
    assert_eq!(find_latest_usage(&five_hundred_one_lines)?, None);

    let malformed_outside = format!("{{broken}}\n{}{}", "\n".repeat(499), usage);
    assert_eq!(latest_usage(&malformed_outside)?.used, 90_000);
    Ok(())
}

#[test]
fn blank_lines_are_ignored_but_retained_malformed_json_is_rejected() -> TestResult {
    assert_eq!(find_latest_usage("\n \t\n")?, None);
    assert_usage_error("{broken}\n", "malformed transcript JSON at retained line 1");
    assert_usage_error(
        "\n{broken}\n",
        "malformed transcript JSON at retained line 2",
    );
    Ok(())
}

#[test]
fn ecmascript_does_not_trim_next_line() {
    let transcript = format!("{}\n\u{0085}\n", claude_usage(r#""input_tokens":42"#));

    assert_usage_error(&transcript, "malformed transcript JSON at retained line 2");
}

#[test]
fn ecmascript_trims_byte_order_mark() -> TestResult {
    let transcript = format!("{}\n\u{feff}\n", claude_usage(r#""input_tokens":42"#));

    assert_eq!(latest_usage(&transcript)?.used, 42);
    Ok(())
}

#[test]
fn unsupported_json_records_do_not_replace_latest_usage() -> TestResult {
    let transcript = format!(
        "{}\nnull\n[]\n{{\"type\":\"other\"}}\n",
        claude_usage(r#""input_tokens":42"#)
    );
    assert_eq!(latest_usage(&transcript)?.used, 42);
    Ok(())
}
