use super::*;

#[test]
fn preserves_thought_hooks_and_similar_measurement_commands() {
    let harness = Harness::new();
    let similar = format!("{} --extra", harness.command("cursor"));
    let excluded = harness.command("cursor");
    harness.write_config(
        "cursor",
        &serde_json::json!({
            "version":1,
            "hooks":{
                "afterAgentThought":[
                    {"command":"third-party-thought"},
                    {"type":"prompt","prompt":excluded},
                    {"command":excluded}
                ],
                "futureEvent":[
                    {"command":excluded},
                    {"command":"third-party-future"}
                ],
                "stop":[{"command":similar}]
            }
        }),
    );

    assert_success(&harness.install("cursor"));

    let config = read_json(harness.config("cursor"));
    assert_eq!(
        config["hooks"]["afterAgentThought"][0]["command"],
        "third-party-thought"
    );
    assert_eq!(config["hooks"]["afterAgentThought"][1]["type"], "prompt");
    assert_eq!(
        config["hooks"]["afterAgentThought"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        config["hooks"]["futureEvent"][0]["command"],
        "third-party-future"
    );
    assert_eq!(config["hooks"]["futureEvent"].as_array().unwrap().len(), 1);
    assert_eq!(config["hooks"]["stop"][0]["command"], similar);
    assert_eq!(
        config["hooks"]["stop"][1]["command"],
        harness.command("cursor")
    );
}

#[test]
fn preserves_prompt_hooks_on_touched_events() {
    let harness = Harness::new();
    harness.write_config(
        "claude-code",
        &serde_json::json!({"hooks":{"Stop":[{"hooks":[
            {"type":"prompt","prompt":"is the task complete?"}
        ]}]}}),
    );
    assert_success(&harness.install("claude-code"));
    let claude = read_json(harness.config("claude-code"));
    assert_eq!(claude["hooks"]["Stop"][0]["hooks"][0]["type"], "prompt");

    let harness = Harness::new();
    harness.write_config(
        "cursor",
        &serde_json::json!({"version":1,"hooks":{"stop":[
            {"type":"prompt","prompt":"is the task complete?"}
        ]}}),
    );
    assert_success(&harness.install("cursor"));
    let cursor = read_json(harness.config("cursor"));
    assert_eq!(cursor["hooks"]["stop"][0]["type"], "prompt");
    assert_eq!(
        cursor["hooks"]["stop"][1]["command"],
        harness.command("cursor")
    );
}
