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

#[test]
fn memory_reconciliation_preserves_third_party_order_and_replaces_owned_handlers() {
    for agent in ["codex", "claude-code"] {
        let harness = Harness::new();
        harness.write_memory_manifest(agent);
        harness.executable_named("agent-memory");
        let command = harness.memory_command(agent);
        harness.write_config(
            agent,
            &serde_json::json!({"hooks":{"UserPromptSubmit":[
                {"matcher":"first","hooks":[{"type":"command","command":"third-before"}]},
                {"hooks":[
                    {"type":"command","command":command,"timeout":7},
                    {"type":"command","command":"third-after"}
                ]},
                {"hooks":[{"type":"command","command":command}]}
            ]}}),
        );

        assert_success(&harness.install(agent));

        let config = read_json(harness.config(agent));
        let commands: Vec<&str> = config["hooks"]["UserPromptSubmit"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["hooks"].as_array().unwrap())
            .filter_map(|handler| handler["command"].as_str())
            .collect();
        assert_eq!(commands, vec!["third-before", "third-after", &command]);
        let owned = config["hooks"]["UserPromptSubmit"]
            .as_array()
            .unwrap()
            .iter()
            .flat_map(|group| group["hooks"].as_array().unwrap())
            .find(|handler| handler["command"] == command)
            .unwrap();
        assert_eq!(owned["timeout"], 30);
    }
}

#[test]
fn owned_memory_hooks_are_removed_when_absent_from_the_manifest() {
    for agent in ["codex", "claude-code"] {
        let harness = Harness::new();
        let command = harness.memory_command(agent);
        harness.write_config(
            agent,
            &serde_json::json!({"hooks":{"UserPromptSubmit":[{"hooks":[
                {"type":"command","command":command},
                {"type":"command","command":"third-party"}
            ]}]}}),
        );

        assert_success(&harness.install(agent));

        let serialized = serde_json::to_string(&read_json(harness.config(agent))).unwrap();
        assert!(!serialized.contains("agent-memory"));
        assert!(serialized.contains("third-party"));
    }
}
