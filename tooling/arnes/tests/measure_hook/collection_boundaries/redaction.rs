use super::super::support::*;

#[test]
fn payload_content_is_never_persisted() {
    let harness = Harness::new();
    let private_values = [
        "fixture-user-prompt",
        "fixture-agent-response",
        "fixture-tool-input",
        "fixture-secret-value",
    ];
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"Stop",
        "prompt":private_values[0],
        "last_assistant_message":private_values[1],
        "tool_input":{"command":private_values[2]},
        "secret":private_values[3]
    });

    assert_success(&harness.run("codex", payload.to_string().as_bytes()));

    let stored = walk(&harness.measure_root())
        .into_iter()
        .filter(|path| path.is_file())
        .flat_map(|path| fs::read(path).unwrap())
        .collect::<Vec<_>>();
    let stored = String::from_utf8_lossy(&stored);
    for value in private_values {
        assert!(!stored.contains(value));
    }
}

#[test]
fn payload_derived_event_and_model_strings_are_not_persisted() {
    let harness = Harness::new();
    let token = "sk-abcdefghijklmnopqrstuvwxyz";
    let payload = json!({
        "conversation_id":"session",
        "hook_event_name":token,
        "model":"private model value",
        "text":"fixture-private-thought"
    });

    assert_success(&harness.run("cursor", payload.to_string().as_bytes()));

    let events = read_jsonl(harness.only_run().join("events.jsonl"));
    assert!(events[0].get("native_event").is_none());
    assert_eq!(
        read_json(harness.only_run().join("run.json"))["model_fingerprint"]
            .as_str()
            .unwrap()
            .len(),
        64
    );
    let stored = walk(&harness.measure_root())
        .into_iter()
        .filter(|path| path.is_file())
        .flat_map(|path| fs::read(path).unwrap())
        .collect::<Vec<_>>();
    let stored = String::from_utf8_lossy(&stored);
    for private in [token, "private model value", "fixture-private-thought"] {
        assert!(!stored.contains(private));
    }
}
