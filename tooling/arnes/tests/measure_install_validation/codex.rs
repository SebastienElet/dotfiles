use super::*;

#[test]
fn codex_known_handler_fields_fail_closed_by_agent_event_and_variant() {
    assert_invalid_codex_handlers(numeric_field_failures());
    assert_invalid_codex_handlers(command_field_failures());
}

fn assert_invalid_codex_handlers(cases: Vec<(&str, &str, Value)>) {
    for (label, event, handler) in cases {
        assert_failure_without_mutation("codex", label, nested(event, handler));
    }
}

fn numeric_field_failures() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "codex Stop command timeout",
            "Stop",
            json!({"type":"command","command":"x","timeout":"30"}),
        ),
        (
            "codex Stop command timeout negative",
            "Stop",
            json!({"type":"command","command":"x","timeout":-1}),
        ),
        (
            "codex Stop command timeout fractional",
            "Stop",
            json!({"type":"command","command":"x","timeout":1.5}),
        ),
        (
            "codex Stop command statusMessage",
            "Stop",
            json!({"type":"command","command":"x","statusMessage":false}),
        ),
        (
            "codex Stop command additionalContextLimit negative",
            "Stop",
            json!({"type":"command","command":"x","additionalContextLimit":-1}),
        ),
        (
            "codex Stop command additionalContextLimit fractional",
            "Stop",
            json!({"type":"command","command":"x","additionalContextLimit":1.5}),
        ),
    ]
}

fn command_field_failures() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "codex Stop command commandWindows",
            "Stop",
            json!({"type":"command","command":"x","commandWindows":false}),
        ),
        (
            "codex Stop command command_windows",
            "Stop",
            json!({"type":"command","command":"x","command_windows":false}),
        ),
        (
            "codex Stop command duplicate Windows aliases",
            "Stop",
            json!({
                "type":"command", "command":"x", "commandWindows":"x.exe",
                "command_windows":"x.exe"
            }),
        ),
        (
            "codex Stop command async",
            "Stop",
            json!({"type":"command","command":"x","async":"yes"}),
        ),
        (
            "codex SessionEnd command timeout range",
            "SessionEnd",
            json!({"type":"command","command":"x","timeout":4}),
        ),
    ]
}
