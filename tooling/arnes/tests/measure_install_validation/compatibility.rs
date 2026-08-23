use super::*;

#[test]
fn documented_fields_and_unknown_extensions_are_preserved() {
    assert_codex_fields_preserved();
    assert_claude_fields_preserved();
    assert_cursor_fields_preserved();
}

fn assert_codex_fields_preserved() {
    let harness = Harness::new();
    harness.write("codex", &codex_documented_fields());
    assert_success(&harness.install("codex"), "codex SessionEnd command valid");
    let codex: Value = serde_json::from_slice(&fs::read(harness.config("codex")).unwrap()).unwrap();
    let hooks = &codex["hooks"]["SessionEnd"][0]["hooks"];
    assert_eq!(hooks[0]["futureField"]["x"], 1);
    assert_eq!(hooks[1]["command_windows"], "other.exe");
}

fn assert_claude_fields_preserved() {
    let harness = Harness::new();
    harness.write("claude-code", &claude_documented_fields());
    assert_success(
        &harness.install("claude-code"),
        "claude Stop all variants valid",
    );
    let claude: Value =
        serde_json::from_slice(&fs::read(harness.config("claude-code")).unwrap()).unwrap();
    let hooks = claude["hooks"]["Stop"][0]["hooks"].as_array().unwrap();
    assert_eq!(hooks[0]["futureField"]["x"], 1);
    assert_eq!(hooks.len(), 5);
}

fn assert_cursor_fields_preserved() {
    let harness = Harness::new();
    harness.write("cursor", &cursor_documented_fields());
    assert_success(
        &harness.install("cursor"),
        "cursor stop command valid object matcher",
    );
    let cursor: Value =
        serde_json::from_slice(&fs::read(harness.config("cursor")).unwrap()).unwrap();
    assert_eq!(cursor["hooks"]["stop"][0]["futureField"]["x"], 1);
    assert_eq!(cursor["version"], 1.5);
}

fn codex_documented_fields() -> Value {
    json!({"hooks":{"SessionEnd":[{"hooks":[
        {
            "type":"command", "command":"third-party", "timeout":3,
            "statusMessage":"done", "additionalContextLimit":0,
            "commandWindows":"third-party.exe", "async":false, "futureField":{"x":1}
        },
        {"type":"command", "command":"other", "command_windows":"other.exe"}
    ]}]}})
}

fn claude_documented_fields() -> Value {
    json!({"hooks":{"Stop":[{"hooks":[
        {
            "type":"command", "command":"third-party", "args":["a"], "async":true,
            "asyncRewake":true, "shell":"bash", "timeout":1, "statusMessage":"running",
            "once":false, "if":"Bash(*)", "futureField":{"x":1}
        },
        {
            "type":"http", "url":"https://example.test", "headers":{"x":"$TOKEN"},
            "allowedEnvVars":["TOKEN"]
        },
        {"type":"mcp_tool", "server":"s", "tool":"t", "input":["opaque"]},
        {"type":"prompt", "prompt":"review", "model":"fast", "continueOnBlock":true},
        {"type":"agent", "prompt":"investigate", "model":"fast"}
    ]}]}})
}

fn cursor_documented_fields() -> Value {
    json!({"version":1.5,"hooks":{"stop":[
        {
            "command":"third-party", "timeout":1, "loop_limit":null,
            "failClosed":true, "matcher":{"type":"Shell"}, "futureField":{"x":1}
        },
        {
            "type":"prompt", "prompt":"review", "model":"fast", "timeout":1,
            "loop_limit":5, "failClosed":false, "matcher":"Stop"
        }
    ]}})
}

#[test]
fn future_event_and_variant_extensions_remain_opaque() {
    for (agent, event, value) in codex_future_extensions()
        .into_iter()
        .chain(other_future_extensions())
    {
        assert_future_extension_preserved(agent, event, value);
    }
}

fn codex_future_extensions() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "codex",
            "FutureEvent",
            nested(
                "FutureEvent",
                json!({"type":"http","timeout":"future","future":true}),
            ),
        ),
        (
            "codex",
            "Stop",
            nested(
                "Stop",
                json!({"type":"http","timeout":"future","future":true}),
            ),
        ),
        (
            "codex",
            "Stop",
            nested(
                "Stop",
                json!({"type":"agent","timeout":"future","future":true}),
            ),
        ),
        (
            "codex",
            "Stop",
            nested(
                "Stop",
                json!({"type":"prompt","command":"future","timeout":"future","future":true}),
            ),
        ),
    ]
}

fn other_future_extensions() -> Vec<(&'static str, &'static str, Value)> {
    vec![
        (
            "claude-code",
            "FutureEvent",
            nested(
                "FutureEvent",
                json!({"type":"future","timeout":"future","future":true}),
            ),
        ),
        (
            "claude-code",
            "Stop",
            nested(
                "Stop",
                json!({"type":"future","timeout":"future","future":true}),
            ),
        ),
        (
            "cursor",
            "futureEvent",
            direct(
                "futureEvent",
                json!({"type":"future","timeout":"future","future":true}),
            ),
        ),
        (
            "cursor",
            "stop",
            direct(
                "stop",
                json!({"type":"future","timeout":"future","future":true}),
            ),
        ),
    ]
}

fn assert_future_extension_preserved(agent: &str, event: &str, value: Value) {
    let harness = Harness::new();
    harness.write(agent, &value);
    assert_success(&harness.install(agent), agent);
    let installed: Value =
        serde_json::from_slice(&fs::read(harness.config(agent)).unwrap()).unwrap();
    let handler = match agent {
        "cursor" => &installed["hooks"][event][0],
        _ => &installed["hooks"][event][0]["hooks"][0],
    };
    assert_eq!(handler["timeout"], "future");
    assert_eq!(handler["future"], true);
}
