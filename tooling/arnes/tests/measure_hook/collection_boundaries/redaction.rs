use super::super::support::*;
#[test]
fn payload_content_is_never_persisted() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let private_values = [
        "fixture-user-prompt",
        "fixture-agent-response",
        "fixture-tool-input",
        "fixture-secret-value",
    ];
    let payload = json ! ({ "session_id" : "session" , "hook_event_name" : "Stop" , "prompt" : private_values [0] , "last_assistant_message" : private_values [1] , "tool_input" : { "command" : private_values [2] } , "secret" : private_values [3] });
    assert_success(&harness.run("codex", payload.to_string().as_bytes())?);
    let stored = walk(&harness.measure_root())?
        .into_iter()
        .filter(|path| path.is_file())
        .map(|path| -> Result<_, Box<dyn std::error::Error + Send + Sync>> { Ok(fs::read(path)?) })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let stored = String::from_utf8_lossy(&stored);
    let _: () = for value in private_values {
        assert!(!stored.contains(value));
    };
    Ok(())
}
#[test]
fn payload_derived_event_and_model_strings_are_not_persisted()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let token = "sk-abcdefghijklmnopqrstuvwxyz";
    let payload = json ! ({ "conversation_id" : "session" , "hook_event_name" : token , "model" : "private model value" , "text" : "fixture-private-thought" });
    assert_success(&harness.run("cursor", payload.to_string().as_bytes())?);
    let events = read_jsonl(harness.only_run()?.join("events.jsonl"))?;
    assert!(
        (*(events).first().ok_or("missing fixture index 0")?)
            .get("native_event")
            .is_none()
    );
    assert_eq!(
        (*(read_json(harness.only_run()?.join("run.json"))?)
            .get("model_fingerprint")
            .ok_or("missing fixture index model_fingerprint")?)
        .as_str()
        .ok_or("expected JSON string")?
        .len(),
        64
    );
    let stored = walk(&harness.measure_root())?
        .into_iter()
        .filter(|path| path.is_file())
        .map(|path| -> Result<_, Box<dyn std::error::Error + Send + Sync>> { Ok(fs::read(path)?) })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
    let stored = String::from_utf8_lossy(&stored);
    let _: () = for private in [token, "private model value", "fixture-private-thought"] {
        assert!(!stored.contains(private));
    };
    Ok(())
}
