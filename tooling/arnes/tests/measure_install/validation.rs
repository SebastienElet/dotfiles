use super::*;

#[test]
fn malformed_touched_hook_structures_are_rejected_without_mutation() {
    let malformed = [
        "",
        "null",
        "false",
        "[]",
        r#"{"hooks":false}"#,
        r#"{"hooks":{"Stop":false}}"#,
        r#"{"hooks":{"Stop":[false]}}"#,
        r#"{"hooks":{"Stop":[{}]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":false}]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":[false]}]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":[{}]}]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":[{"command":"missing-type"}]}]}}"#,
        r#"{"hooks":{"Stop":[{"matcher":1,"hooks":[]}]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":[{"command":1}]}]}}"#,
        r#"{"hooks":{"Stop":[]}} {"second":true}"#,
    ];
    for raw in malformed {
        let harness = Harness::new();
        let path = harness.config("codex");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, raw).unwrap();
        let before = fs::read(&path).unwrap();
        assert_failure(&harness.install("codex"));
        assert_eq!(fs::read(path).unwrap(), before, "accepted {raw}");
    }

    for raw in [
        r#"{"version":"one","hooks":{}}"#,
        r#"{"version":1,"hooks":{"stop":false}}"#,
        r#"{"version":1,"hooks":{"stop":[false]}}"#,
        r#"{"version":1,"hooks":{"stop":[{}]}}"#,
        r#"{"version":1,"hooks":{"stop":[{"command":1}]}}"#,
        r#"{"version":1,"hooks":{"stop":[{"type":"prompt"}]}}"#,
        r#"{"version":1,"hooks":{"stop":[{"command":"ok","matcher":1}]}}"#,
    ] {
        let harness = Harness::new();
        let path = harness.config("cursor");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, raw).unwrap();
        let before = fs::read(&path).unwrap();
        assert_failure(&harness.install("cursor"));
        assert_eq!(fs::read(path).unwrap(), before, "accepted {raw}");
    }
}

#[test]
fn incomplete_recognized_hook_handlers_are_rejected_without_mutation() {
    for (agent, raw) in [
        (
            "claude-code",
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"command"}]}]}}"#,
        ),
        (
            "claude-code",
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"http"}]}]}}"#,
        ),
        (
            "claude-code",
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"mcp_tool","server":"s"}]}]}}"#,
        ),
        (
            "claude-code",
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"prompt"}]}]}}"#,
        ),
        (
            "claude-code",
            r#"{"hooks":{"Stop":[{"hooks":[{"type":"agent"}]}]}}"#,
        ),
    ] {
        let harness = Harness::new();
        let path = harness.config(agent);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, raw).unwrap();
        assert_failure(&harness.install(agent));
        assert_eq!(fs::read_to_string(path).unwrap(), raw, "accepted {raw}");
    }
}

#[test]
fn preserves_all_documented_third_party_handler_types() {
    assert_handler_count("codex", codex_handler_types(), "Stop", 2);
    assert_handler_count("claude-code", claude_handler_types(), "Stop", 5);
    assert_handler_count("cursor", cursor_handler_types(), "stop", 4);
}

fn assert_handler_count(agent: &str, config: Value, event: &str, expected: usize) {
    let harness = Harness::new();
    harness.write_config(agent, &config);
    assert_success(&harness.install(agent));
    let installed = read_json(harness.config(agent));
    let handlers = if agent == "cursor" {
        installed["hooks"][event].as_array().unwrap()
    } else {
        installed["hooks"][event][0]["hooks"].as_array().unwrap()
    };
    assert_eq!(handlers.len(), expected);
}

fn codex_handler_types() -> Value {
    serde_json::json!({"hooks":{"Stop":[{"hooks":[
        {"type":"prompt","prompt":"review"},
        {"type":"agent","prompt":"investigate"}
    ]}]}})
}

fn claude_handler_types() -> Value {
    serde_json::json!({"hooks":{"Stop":[{"hooks":[
            {"type":"command","command":"third-party"},
            {"type":"http","url":"https://example.com/hook"},
            {"type":"mcp_tool","server":"review","tool":"record","input":{"ok":true}},
            {"type":"prompt","prompt":"review"},
            {"type":"agent","prompt":"investigate"}
    ]}]}})
}

fn cursor_handler_types() -> Value {
    serde_json::json!({"version":1,"hooks":{"stop":[
        {"command":"third-party"},
        {"type":"command","command":"typed-third-party"},
        {"type":"prompt","prompt":"review"}
    ]}})
}

#[test]
fn duplicate_json_keys_are_rejected_without_mutation() {
    for raw in [
        r#"{"hooks":{},"hooks":{"Stop":[]}}"#,
        r#"{"hooks":{"Stop":[],"Stop":[]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":[],"hooks":[]}]}}"#,
        r#"{"hooks":{"Stop":[{"hooks":[{"command":"a","command":"b"}]}]}}"#,
    ] {
        let harness = Harness::new();
        let config = harness.config("codex");
        fs::create_dir_all(config.parent().unwrap()).unwrap();
        fs::write(&config, raw).unwrap();
        assert_failure(&harness.install("codex"));
        assert_eq!(fs::read_to_string(config).unwrap(), raw);
    }
}
