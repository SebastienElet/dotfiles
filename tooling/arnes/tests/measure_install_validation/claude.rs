use super::*;

#[test]
fn claude_known_handler_fields_fail_closed_by_agent_event_and_variant() {
    for (label, event, handler) in [
        (
            "claude Stop command timeout",
            "Stop",
            json!({"type":"command","command":"x","timeout":"30"}),
        ),
        (
            "claude Stop command statusMessage",
            "Stop",
            json!({"type":"command","command":"x","statusMessage":false}),
        ),
        (
            "claude Stop command once",
            "Stop",
            json!({"type":"command","command":"x","once":"yes"}),
        ),
        (
            "claude Stop command if",
            "Stop",
            json!({"type":"command","command":"x","if":false}),
        ),
        (
            "claude Stop command args",
            "Stop",
            json!({"type":"command","command":"x","args":["ok",1]}),
        ),
        (
            "claude Stop command async",
            "Stop",
            json!({"type":"command","command":"x","async":"yes"}),
        ),
        (
            "claude Stop command asyncRewake",
            "Stop",
            json!({"type":"command","command":"x","asyncRewake":"yes"}),
        ),
        (
            "claude Stop command shell",
            "Stop",
            json!({"type":"command","command":"x","shell":"zsh"}),
        ),
        (
            "claude Stop http headers",
            "Stop",
            json!({"type":"http","url":"https://example.test","headers":{"x":1}}),
        ),
        (
            "claude Stop http allowedEnvVars",
            "Stop",
            json!({"type":"http","url":"https://example.test","allowedEnvVars":["A",1]}),
        ),
        (
            "claude Stop prompt model",
            "Stop",
            json!({"type":"prompt","prompt":"x","model":1}),
        ),
        (
            "claude Stop prompt continueOnBlock",
            "Stop",
            json!({"type":"prompt","prompt":"x","continueOnBlock":"yes"}),
        ),
        (
            "claude Stop agent continueOnBlock incompatible",
            "Stop",
            json!({"type":"agent","prompt":"x","continueOnBlock":true}),
        ),
        (
            "claude Stop command url incompatible",
            "Stop",
            json!({"type":"command","command":"x","url":"https://example.test"}),
        ),
        (
            "claude SessionEnd command timeout range",
            "SessionEnd",
            json!({"type":"command","command":"x","timeout":61}),
        ),
        (
            "claude ConfigChange prompt matrix",
            "ConfigChange",
            json!({"type":"prompt","prompt":"x"}),
        ),
    ] {
        assert_failure_without_mutation("claude-code", label, nested(event, handler));
    }
}

#[test]
fn claude_event_variant_matrix_fails_closed_and_accepts_documented_pairs() {
    for (label, event, handler) in [
        (
            "claude SessionStart prompt",
            "SessionStart",
            json!({"type":"prompt","prompt":"x"}),
        ),
        (
            "claude SessionStart http",
            "SessionStart",
            json!({"type":"http","url":"https://example.test"}),
        ),
        (
            "claude PreCompact prompt",
            "PreCompact",
            json!({"type":"prompt","prompt":"x"}),
        ),
    ] {
        assert_failure_without_mutation("claude-code", label, nested(event, handler));
    }

    for (label, event, handler) in [
        (
            "claude SessionStart command",
            "SessionStart",
            json!({"type":"command","command":"x"}),
        ),
        (
            "claude SessionStart mcp_tool",
            "SessionStart",
            json!({"type":"mcp_tool","server":"s","tool":"t"}),
        ),
        (
            "claude PreCompact http",
            "PreCompact",
            json!({"type":"http","url":"https://example.test"}),
        ),
        (
            "claude Stop agent",
            "Stop",
            json!({"type":"agent","prompt":"x"}),
        ),
    ] {
        let harness = Harness::new();
        harness.write("claude-code", &nested(event, handler));
        assert_success(&harness.install("claude-code"), label);
    }
}
