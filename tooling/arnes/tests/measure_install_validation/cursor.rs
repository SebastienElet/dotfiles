use super::*;

#[test]
fn cursor_known_handler_fields_fail_closed_by_agent_event_and_variant() {
    assert_invalid_cursor_handlers(common_field_failures());
    assert_invalid_cursor_handlers(incompatible_field_failures());
}

fn assert_invalid_cursor_handlers(cases: Vec<(&str, &str, Value)>) {
    for (label, event, handler) in cases {
        assert_failure_without_mutation("cursor", label, direct(event, handler));
    }
}

fn common_field_failures() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "cursor stop command timeout",
            "stop",
            json!({"command":"x","timeout":"30"}),
        ),
        (
            "cursor stop command failClosed",
            "stop",
            json!({"command":"x","failClosed":"yes"}),
        ),
        (
            "cursor stop command loop_limit",
            "stop",
            json!({"command":"x","loop_limit":false}),
        ),
        (
            "cursor stop command matcher",
            "stop",
            json!({"command":"x","matcher":[]}),
        ),
        (
            "cursor stop prompt model",
            "stop",
            json!({"type":"prompt","prompt":"x","model":1}),
        ),
    ]
}

fn incompatible_field_failures() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "cursor stop prompt command incompatible",
            "stop",
            json!({"type":"prompt","prompt":"x","command":"x"}),
        ),
        (
            "cursor stop command prompt incompatible",
            "stop",
            json!({"type":"command","command":"x","prompt":"x"}),
        ),
        (
            "cursor preToolUse command loop_limit incompatible",
            "preToolUse",
            json!({"command":"x","loop_limit":5}),
        ),
        (
            "cursor beforeReadFile command failClosed",
            "beforeReadFile",
            json!({"command":"x","failClosed":"yes"}),
        ),
        (
            "cursor postCompact command timeout",
            "postCompact",
            json!({"command":"x","timeout":"30"}),
        ),
    ]
}

#[test]
fn cursor_accepts_both_documented_matcher_shapes() {
    for (label, matcher) in [
        (
            "cursor preToolUse command string matcher",
            json!("Shell|Read"),
        ),
        (
            "cursor preToolUse command object matcher",
            json!({"type":"Shell"}),
        ),
    ] {
        let harness = Harness::new();
        harness.write(
            "cursor",
            &direct("preToolUse", json!({"command":"x","matcher":matcher})),
        );
        assert_success(&harness.install("cursor"), label);
    }
}
