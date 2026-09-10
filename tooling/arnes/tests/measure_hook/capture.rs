use super::support::*;
#[test]
fn accepts_exact_agent_names_and_native_session_keys()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (agent, payload) in [
        (
            "codex",
            json ! ({ "session_id" : "codex-session" , "event" : "SessionStart" }),
        ),
        (
            "claude-code",
            json ! ({ "session_id" : "claude-session" , "hook_event_name" : "SessionStart" }),
        ),
        (
            "cursor",
            json ! ({ "conversation_id" : "cursor-session" , "hook_event_name" : "sessionStart" }),
        ),
    ] {
        let harness = Harness::new()?;
        assert_success(&harness.run(agent, payload.to_string().as_bytes())?);
        let run = read_json(harness.only_run()?.join("run.json"))?;
        assert_eq!(
            *(run)
                .get("schema_version")
                .ok_or("missing fixture index schema_version")?,
            2
        );
        assert_eq!(
            *(run).get("agent").ok_or("missing fixture index agent")?,
            agent
        );
        assert!(run.get("session_id").is_none());
        assert!(run.get("repository").is_none());
        assert!(run.get("repository_branch").is_none());
        assert_eq!(
            (*(run).get("run_id").ok_or("missing fixture index run_id")?)
                .as_str()
                .ok_or("expected JSON string")?
                .len(),
            64
        );
        assert!(
            (*(run)
                .get("started_at_ms")
                .ok_or("missing fixture index started_at_ms")?)
            .as_u64()
            .ok_or("expected JSON unsigned integer")?
                > 0
        );
        assert!(run.get("model").is_none());
        assert!(
            (*(run)
                .get("model_fingerprint")
                .ok_or("missing fixture index model_fingerprint")?)
            .is_null()
        );
        assert_eq!(
            (*(run)
                .get("harness_fingerprint")
                .ok_or("missing fixture index harness_fingerprint")?)
            .as_str()
            .ok_or("expected JSON string")?
            .len(),
            64
        );
        assert_eq!(
            *(run)
                .get("operating_system")
                .ok_or("missing fixture index operating_system")?,
            std::env::consts::OS
        );
        assert_eq!(
            *(run)
                .get("architecture")
                .ok_or("missing fixture index architecture")?,
            std::env::consts::ARCH
        );
    };
    Ok(())
}
#[test]
fn stores_only_compact_measurement_data() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = json ! ({ "session_id" : "compact-session" , "hook_event_name" : "UserPromptSubmit" , "prompt" : "private fixture value" , "message_id" : "native-message" , "future" : { "answer" : 42 } });
    assert_success(&harness.run("codex", payload.to_string().as_bytes())?);
    let run = harness.only_run()?;
    assert!(!run.join("artifacts").exists());
    assert!(!run.join("prompts.jsonl").exists());
    let event = read_jsonl(run.join("events.jsonl"))?.remove(0);
    assert_eq!(event.as_object().ok_or("expected JSON object")?.len(), 3);
    assert_eq!(
        *(event)
            .get("schema_version")
            .ok_or("missing fixture index schema_version")?,
        2
    );
    assert!(
        (*(event)
            .get("timestamp_ms")
            .ok_or("missing fixture index timestamp_ms")?)
        .as_u64()
        .ok_or("expected JSON unsigned integer")?
            > 0
    );
    assert_eq!(
        *(event).get("event").ok_or("missing fixture index event")?,
        "prompt.submit"
    );
    assert!(event.get("native_event").is_none());
    Ok(())
}
#[test]
fn rejects_every_other_agent_name() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let _: () = for agent in ["claude", "Claude-Code", "cursor-agent", "codex "] {
        let output = harness.run(agent, br#"{"session_id":"session"}"#)?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr)?;
        assert!(stderr.contains("invalid value"), "{stderr}");
    };
    Ok(())
}
#[test]
fn appends_compact_events_without_prompt_or_native_identifiers()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let first = json ! ({ "session_id" : "session" , "hook_event_name" : "UserPromptSubmit" , "prompt" : "first prompt" , "message_id" : "message-one" });
    let second = json ! ({ "session_id" : "session" , "hook_event_name" : "UserPromptSubmit" , "prompt" : "followup prompt" , "message_id" : "message-two" });
    assert_success(&harness.run("claude-code", first.to_string().as_bytes())?);
    assert_success(&harness.run("claude-code", second.to_string().as_bytes())?);
    let run = harness.only_run()?;
    let events = read_jsonl(run.join("events.jsonl"))?;
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| event.get("native_ids").is_none()));
    assert!(!run.join("prompts.jsonl").exists());
    Ok(())
}
#[test]
fn preserves_unknown_events_without_the_payload_event_name()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = json ! ({ "session_id" : "session" , "hook_event_name" : "FutureEvent" , "future" : { "answer" : 42 } , "event_id" : "native-event" });
    assert_success(&harness.run("codex", payload.to_string().as_bytes())?);
    let run = harness.only_run()?;
    let events = read_jsonl(run.join("events.jsonl"))?;
    assert_eq!(events.len(), 1);
    assert_eq!(
        *(*(events).first().ok_or("missing fixture index 0")?)
            .get("event")
            .ok_or("missing fixture index event")?,
        "unknown"
    );
    assert!(
        (*(events).first().ok_or("missing fixture index 0")?)
            .get("native_event")
            .is_none()
    );
    assert!(
        (*(events).first().ok_or("missing fixture index 0")?)
            .get("native_ids")
            .is_none()
    );
    assert!(
        (*(events).first().ok_or("missing fixture index 0")?)
            .get("artifact")
            .is_none()
    );
    assert!(!run.join("artifacts").exists());
    Ok(())
}
#[test]
fn large_payload_is_capturable_without_persisting_its_size()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = json ! ({ "session_id" : "compact-large" , "hook_event_name" : "FutureEvent" , "data" : vec ! [0 ; 250_000] });
    let compact = serde_json::to_vec(&payload)?;
    assert!(compact.len() < 1_048_576);
    assert!(serde_json::to_vec_pretty(&payload)?.len() > 1_100_000);
    assert_success(&harness.run("codex", &compact)?);
    let run = harness.only_run()?;
    assert!(!run.join("artifacts").exists());
    assert!(fs::metadata(run.join("events.jsonl"))?.len() < 256);
    let listed = harness.list()?;
    assert_eq!(listed.status.code(), Some(0));
    assert!(listed.stderr.is_empty());
    let listed: Value = serde_json::from_slice(&listed.stdout)?;
    assert_eq!(listed.as_array().ok_or("expected JSON array")?.len(), 1);
    Ok(())
}
#[test]
fn expanded_payload_representation_does_not_create_a_storage_failure()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = json ! ({ "session_id" : "expanded-large" , "hook_event_name" : "FutureEvent" , "data" : vec ! [json ! ({ "token" : "x" }) ; 65_000] });
    let compact = serde_json::to_vec(&payload)?;
    assert!(compact.len() < 1_048_576);
    assert_success(&harness.run("codex", &compact)?);
    assert_eq!(
        read_jsonl(harness.only_run()?.join("events.jsonl"))?.len(),
        1
    );
    assert!(!harness.measure_root().join("invalid.jsonl").exists());
    Ok(())
}
