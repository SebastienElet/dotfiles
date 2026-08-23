use super::*;

#[test]
fn installs_and_migrates_the_owned_claude_stop_hook_in_the_same_write() {
    let harness = Harness::new();
    let current = harness.home.join("agent-handoff");
    let legacy = harness.home.join("agent_handoff");
    fs::write(&current, b"binary").unwrap();
    fs::set_permissions(&current, fs::Permissions::from_mode(0o700)).unwrap();
    harness.write_config(
        "claude-code",
        &serde_json::json!({"keep":true,"hooks":{"Stop":[{"hooks":[
            {
                "type":"command",
                "command":legacy,
                "timeout":7,
                "async":true,
                "asyncRewake":true,
                "once":true,
                "if":"never"
            },
            {"type":"command","command":"third-party"}
        ]}]}}),
    );

    assert_success(&harness.install_claude_with_handoff(&current, &legacy));

    let config = read_json(harness.config("claude-code"));
    let serialized = serde_json::to_string(&config).unwrap();
    assert_eq!(config["keep"], true);
    assert!(!serialized.contains(legacy.to_str().unwrap()));
    assert_eq!(serialized.matches(current.to_str().unwrap()).count(), 1);
    assert!(serialized.contains("third-party"));
    let owned = config["hooks"]["Stop"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["hooks"].as_array().unwrap())
        .find(|handler| handler["command"] == current.to_str().unwrap())
        .unwrap();
    assert_eq!(owned["args"], serde_json::json!([]));
    assert_eq!(owned["timeout"], 7);
    for field in ["async", "asyncRewake", "once", "if"] {
        assert!(owned.get(field).is_none(), "{field} was preserved");
    }
}
