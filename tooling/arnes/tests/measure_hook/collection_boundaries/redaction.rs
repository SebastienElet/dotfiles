use super::super::support::*;

#[test]
fn recursively_redacts_secret_keys_tokens_and_internal_reasoning() {
    let harness = Harness::new();
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"UserPromptSubmit sk-eventnamesecret",
        "model":"sk-modelnamesecret",
        "prompt":"use Bearer abc.def.ghi, sk-abcdefghijklmnopqrstuvwxyz, AKIAABCDEFGHIJKLMNOP, and glpat-abcdefghijklmnop",
        "api_key":"plain-secret",
        "oauth":{"client_secret":"nested-client-secret"},
        "aws_secret_access_key":"aws-secret-value",
        "reasoning_content":"reasoning detail",
        "analysis":"analysis detail",
        "request_id":"sk-requestidentifiersecret",
        "nested":{"password":"hunter2","reasoning":"private chain"},
        "safe":"visible"
    });
    assert_success(&harness.run("codex", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let events = read_jsonl(run.join("events.jsonl"));
    let artifact = fs::read_to_string(run.join(events[0]["artifact"].as_str().unwrap())).unwrap();
    let normalized = fs::read_to_string(run.join("events.jsonl")).unwrap();
    let prompts = fs::read_to_string(run.join("prompts.jsonl")).unwrap();
    let run_record = fs::read_to_string(run.join("run.json")).unwrap();
    for secret in [
        "abc.def.ghi",
        "sk-abcdefghijklmnopqrstuvwxyz",
        "AKIAABCDEFGHIJKLMNOP",
        "glpat-abcdefghijklmnop",
        "plain-secret",
        "nested-client-secret",
        "aws-secret-value",
        "reasoning detail",
        "analysis detail",
        "sk-requestidentifiersecret",
        "sk-modelnamesecret",
        "sk-eventnamesecret",
        "hunter2",
        "private chain",
    ] {
        assert!(!artifact.contains(secret), "artifact leaked {secret}");
        assert!(!normalized.contains(secret), "event log leaked {secret}");
        assert!(!prompts.contains(secret), "prompt log leaked {secret}");
        assert!(!run_record.contains(secret), "run record leaked {secret}");
    }
    assert!(artifact.contains("visible"));
    assert!(artifact.contains("[REDACTED]"));
}

#[test]
fn prompt_redaction_preserves_words_and_masks_plausible_tokens() {
    let harness = Harness::new();
    let prompt = "task-specific risk-based sk-short sk-abcdefghijklmnopqrstuvwxyz";
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"UserPromptSubmit",
        "prompt":prompt
    });

    assert_success(&harness.run("codex", payload.to_string().as_bytes()));

    let prompts = read_jsonl(harness.only_run().join("prompts.jsonl"));
    assert_eq!(
        prompts[0]["prompt"],
        "task-specific risk-based sk-short [REDACTED]"
    );
}

#[test]
fn after_agent_thought_never_persists_reasoning_values() {
    let harness = Harness::new();
    let secret = "manually-injected-private-thought";
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"afterAgentThought",
        "text":secret,
        "prompt":secret,
        "unclassified":secret,
        "thought":secret,
        "nested":{
            "thoughts":secret,
            "reasoning":secret,
            "chain_of_thought":secret
        }
    });

    assert_success(&harness.run("claude-code", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let artifact = fs::read_to_string(
        fs::read_dir(run.join("artifacts/hooks"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path(),
    )
    .unwrap();
    assert!(!artifact.contains(secret));
    assert!(!run.join("prompts.jsonl").exists());
}

#[test]
fn after_agent_response_preserves_non_reasoning_text() {
    let harness = Harness::new();
    let text = "visible-agent-response";
    let payload = json!({
        "conversation_id":"session",
        "hook_event_name":"afterAgentResponse",
        "text":text
    });

    assert_success(&harness.run("cursor", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let artifact = fs::read_to_string(
        fs::read_dir(run.join("artifacts/hooks"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path(),
    )
    .unwrap();
    assert!(artifact.contains(text));
}

#[test]
fn any_thought_discriminator_neutralizes_contradictory_payloads() {
    for payload in [
        json!({
            "session_id":"session",
            "hook_event_name":"afterAgentThought",
            "event":"afterAgentResponse",
            "type":"afterAgentResponse",
            "text":"private-thought"
        }),
        json!({
            "session_id":"session",
            "hook_event_name":"afterAgentResponse",
            "event":"afterAgentThought",
            "type":"afterAgentResponse",
            "text":"private-thought"
        }),
        json!({
            "session_id":"session",
            "hook_event_name":"afterAgentResponse",
            "event":"afterAgentResponse",
            "type":"afterAgentThought",
            "text":"private-thought"
        }),
    ] {
        let harness = Harness::new();
        assert_success(&harness.run("claude-code", payload.to_string().as_bytes()));
        let stored = walk(&harness.only_run())
            .into_iter()
            .filter(|path| path.is_file())
            .flat_map(|path| fs::read(path).unwrap())
            .collect::<Vec<_>>();
        assert!(!String::from_utf8_lossy(&stored).contains("private-thought"));
    }
}
