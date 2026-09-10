use super::support::*;
#[test]
fn normalizes_cross_agent_event_names() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (agent, session_key, native_events) in [
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
        let harness = Harness::new()?;
        for native_event in native_events {
            let mut payload = json ! ({ "hook_event_name" : native_event });
            payload
                .as_object_mut()
                .ok_or("expected hook payload object")?
                .insert(session_key.to_owned(), json!("session"));
            assert_success(&harness.run(agent, payload.to_string().as_bytes())?);
        }
        let events = read_jsonl(harness.only_run()?.join("events.jsonl"))?;
        let normalized: Vec<&str> = events
            .iter()
            .map(
                |event| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(
                        (*(event).get("event").ok_or("missing fixture index event")?)
                            .as_str()
                            .ok_or("expected JSON string")?,
                    )
                },
            )
            .collect::<Result<Vec<_>, _>>()?;
        assert_eq!(
            normalized,
            [
                "session.start",
                "prompt.submit",
                "agent.stop",
                "subagent.stop"
            ]
        );
        assert!(
            events
                .iter()
                .all(|event| event.get("native_event").is_none())
        );
    };
    Ok(())
}
#[test]
fn normalizes_every_installed_failure_and_compaction_event()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (agent, session_key, native_event, expected) in [
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
        let harness = Harness::new()?;
        let mut payload = json ! ({ "hook_event_name" : native_event });
        payload
            .as_object_mut()
            .ok_or("expected hook payload object")?
            .insert(session_key.to_owned(), json!("session"));
        assert_success(&harness.run(agent, payload.to_string().as_bytes())?);
        let events = read_jsonl(harness.only_run()?.join("events.jsonl"))?;
        assert_eq!(
            *(*(events).first().ok_or("missing fixture index 0")?)
                .get("event")
                .ok_or("missing fixture index event")?,
            expected,
            "{agent} {native_event}"
        );
    };
    Ok(())
}
