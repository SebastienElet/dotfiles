use super::*;

const COMPLETED: &str = r#"{"type":"turn.completed","usage":{"input_tokens":4,"cached_input_tokens":1,"output_tokens":2}}"#;

#[test]
fn reads_usage_and_deduplicates_completed_tool_calls() -> Result<(), Box<dyn std::error::Error>> {
    let output = format!(
        "\n{COMPLETED}\n{}\n{}\n{}\n",
        r#"{"type":"item.completed","item":{"id":"one","type":"command_execution"}}"#,
        r#"{"type":"item.completed","item":{"id":"one","type":"command_execution"}}"#,
        r#"{"type":"item.completed","item":{"id":"two","type":"agent_message"}}"#
    );
    let result = parse_codex_events(&output)?;
    assert_eq!(
        result.tokens,
        Tokens {
            input: 4,
            cached_input: 1,
            output: 2
        }
    );
    assert_eq!(result.tool_calls, 1);
    Ok(())
}

#[test]
fn rejects_failed_malformed_missing_or_multiple_completed_turns() {
    for output in [String::new(), "not-json".into(), "{}".into(), format!("{COMPLETED}\n{COMPLETED}"), format!("{COMPLETED}\n{}", r#"{"type":"error"}"#), format!("{COMPLETED}\n{}", r#"{"type":"turn.failed"}"#), r#"{"type":"turn.completed","usage":{"input_tokens":-1,"cached_input_tokens":0,"output_tokens":2}}"#.into(), format!("{COMPLETED}\n{}", r#"{"type":"item.completed","item":{"id":2,"type":"command_execution"}}"#)] {
        assert!(parse_codex_events(&output).is_err(), "{output}");
    }
}

#[test]
fn rejects_usage_outside_original_safe_integer_domain() {
    for input in ["9007199254740992", "1.5"] {
        let output = format!(
            r#"{{"type":"turn.completed","usage":{{"input_tokens":{input},"cached_input_tokens":0,"output_tokens":2}}}}"#
        );
        assert!(parse_codex_events(&output).is_err());
    }
}

#[test]
fn accepts_json_number_encodings_of_safe_integers() -> Result<(), Box<dyn std::error::Error>> {
    let output = r#"{"type":"turn.completed","usage":{"input_tokens":4.0,"cached_input_tokens":1e0,"output_tokens":-0}}"#;
    let metrics = parse_codex_events(output)?;
    assert_eq!(
        metrics.tokens,
        Tokens {
            input: 4,
            cached_input: 1,
            output: 0
        }
    );
    Ok(())
}
