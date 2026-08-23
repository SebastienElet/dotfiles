use super::support::*;

#[test]
fn normalizes_cross_agent_event_names_and_preserves_native_names() {
    for (agent, session_key, native_events) in [
        (
            "codex",
            "session_id",
            ["SessionStart", "UserPromptSubmit", "Stop", "SubagentStop"],
        ),
        (
            "claude-code",
            "session_id",
            ["SessionStart", "UserPromptSubmit", "Stop", "SubagentStop"],
        ),
        (
            "cursor",
            "conversation_id",
            ["sessionStart", "beforeSubmitPrompt", "stop", "subagentStop"],
        ),
    ] {
        let harness = Harness::new();
        for native_event in native_events {
            let mut payload = json!({"hook_event_name":native_event});
            payload[session_key] = json!("session");
            assert_success(&harness.run(agent, payload.to_string().as_bytes()));
        }
        let events = read_jsonl(harness.only_run().join("events.jsonl"));
        let normalized: Vec<&str> = events
            .iter()
            .map(|event| event["event"].as_str().unwrap())
            .collect();
        let native: Vec<&str> = events
            .iter()
            .map(|event| event["native_event"].as_str().unwrap())
            .collect();
        assert_eq!(
            normalized,
            [
                "session.start",
                "prompt.submit",
                "agent.stop",
                "subagent.stop"
            ]
        );
        assert_eq!(native, native_events);
    }
}

#[test]
fn normalizes_every_installed_failure_and_compaction_event() {
    for (agent, session_key, native_event, expected) in [
        (
            "codex",
            "session_id",
            "PreCompact",
            "context.compact.before",
        ),
        (
            "codex",
            "session_id",
            "PostCompact",
            "context.compact.after",
        ),
        (
            "claude-code",
            "session_id",
            "PermissionDenied",
            "permission.denied",
        ),
        (
            "claude-code",
            "session_id",
            "PostToolUseFailure",
            "tool.failure",
        ),
        ("claude-code", "session_id", "StopFailure", "agent.failure"),
        (
            "cursor",
            "conversation_id",
            "postToolUseFailure",
            "tool.failure",
        ),
        (
            "cursor",
            "conversation_id",
            "preCompact",
            "context.compact.before",
        ),
    ] {
        let harness = Harness::new();
        let mut payload = json!({"hook_event_name":native_event});
        payload[session_key] = json!("session");

        assert_success(&harness.run(agent, payload.to_string().as_bytes()));

        let events = read_jsonl(harness.only_run().join("events.jsonl"));
        assert_eq!(events[0]["event"], expected, "{agent} {native_event}");
    }
}
