use super::support::*;

#[test]
fn accepts_exact_agent_names_and_native_session_keys() {
    for (agent, payload, session) in [
        (
            "codex",
            json!({"session_id":"codex-session","event":"SessionStart"}),
            "codex-session",
        ),
        (
            "claude-code",
            json!({"session_id":"claude-session","hook_event_name":"SessionStart"}),
            "claude-session",
        ),
        (
            "cursor",
            json!({"conversation_id":"cursor-session","hook_event_name":"sessionStart"}),
            "cursor-session",
        ),
    ] {
        let harness = Harness::new();
        assert_success(&harness.run(agent, payload.to_string().as_bytes()));
        let run = read_json(harness.only_run().join("run.json"));
        assert_eq!(run["schema_version"], 1);
        assert_eq!(run["agent"], agent);
        assert_eq!(run["session_id"], session);
        assert_eq!(run["run_id"].as_str().unwrap().len(), 64);
        assert!(run["started_at_ms"].as_u64().unwrap() > 0);
        assert!(run["model"].is_null());
        assert_eq!(run["harness_fingerprint"].as_str().unwrap().len(), 64);
    }
}

#[test]
fn rejects_every_other_agent_name() {
    let harness = Harness::new();
    for agent in ["claude", "Claude-Code", "cursor-agent", "codex "] {
        let output = harness.run(agent, br#"{"session_id":"session"}"#);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("invalid value"), "{stderr}");
    }
}

#[test]
fn appends_first_and_followup_prompts_with_native_ids() {
    let harness = Harness::new();
    let first = json!({
        "session_id":"session",
        "hook_event_name":"UserPromptSubmit",
        "prompt":"first prompt",
        "message_id":"message-one"
    });
    let second = json!({
        "session_id":"session",
        "hook_event_name":"UserPromptSubmit",
        "prompt":"followup prompt",
        "message_id":"message-two"
    });
    assert_success(&harness.run("claude-code", first.to_string().as_bytes()));
    assert_success(&harness.run("claude-code", second.to_string().as_bytes()));

    let prompts = read_jsonl(harness.only_run().join("prompts.jsonl"));
    assert_eq!(prompts.len(), 2);
    assert_eq!(prompts[0]["prompt"], "first prompt");
    assert_eq!(prompts[0]["prompt_id"], "message-one");
    assert_eq!(prompts[1]["prompt"], "followup prompt");
    assert_eq!(prompts[1]["prompt_id"], "message-two");
}

#[test]
fn preserves_unknown_events_and_fields_in_the_redacted_artifact() {
    let harness = Harness::new();
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"FutureEvent",
        "future":{"answer":42},
        "event_id":"native-event"
    });
    assert_success(&harness.run("codex", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let events = read_jsonl(run.join("events.jsonl"));
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "unknown");
    assert_eq!(events[0]["native_event"], "FutureEvent");
    assert_eq!(events[0]["native_ids"]["event_id"], "native-event");
    let artifact = read_json(run.join(events[0]["artifact"].as_str().unwrap()));
    assert_eq!(artifact["future"]["answer"], 42);
}

#[test]
fn compact_large_artifact_remains_capturable_and_listable() {
    let harness = Harness::new();
    let payload = json!({
        "session_id": "compact-large",
        "hook_event_name": "FutureEvent",
        "data": vec![0; 250_000]
    });
    let compact = serde_json::to_vec(&payload).unwrap();
    assert!(compact.len() < 1_048_576);
    assert!(serde_json::to_vec_pretty(&payload).unwrap().len() > 1_100_000);

    assert_success(&harness.run("codex", &compact));

    let run = harness.only_run();
    let artifact = fs::read_dir(run.join("artifacts/hooks"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let artifact = fs::read(artifact).unwrap();
    assert!(artifact.len() <= 1_100_000);
    assert_eq!(serde_json::from_slice::<Value>(&artifact).unwrap(), payload);
    let listed = harness.list();
    assert_eq!(listed.status.code(), Some(0));
    assert!(listed.stderr.is_empty());
    let listed: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
}

#[test]
fn expanded_artifact_is_rejected_without_a_partial_run() {
    let harness = Harness::new();
    let payload = json!({
        "session_id": "expanded-large",
        "hook_event_name": "FutureEvent",
        "data": vec![json!({"token": "x"}); 65_000]
    });
    let compact = serde_json::to_vec(&payload).unwrap();
    assert!(compact.len() < 1_048_576);

    let output = harness.run("codex", &compact);

    assert_advisory_failure(&output);
    assert!(harness.runs().is_empty());
    let invalid = read_jsonl(harness.measure_root().join("invalid.jsonl"));
    assert_eq!(invalid.len(), 1);
    assert!(
        invalid[0]["error"]
            .as_str()
            .unwrap()
            .contains("exceeds 1100000 bytes")
    );
}
