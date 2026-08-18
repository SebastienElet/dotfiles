use serde_json::{Value, json};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::{Command, Output};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
    executable: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let executable = root.path().join("bin/arnes");
        fs::create_dir(&home).unwrap();
        fs::create_dir(executable.parent().unwrap()).unwrap();
        fs::write(&executable, b"binary").unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o700)).unwrap();
        Self {
            _root: root,
            home,
            executable,
        }
    }

    fn config(&self, agent: &str) -> PathBuf {
        self.home.join(match agent {
            "codex" => ".codex/hooks.json",
            "claude-code" => ".claude/settings.json",
            "cursor" => ".cursor/hooks.json",
            _ => unreachable!(),
        })
    }

    fn install(&self, agent: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["measure", "install-hooks", "--agent", agent, "--command"])
            .arg(&self.executable)
            .env_clear()
            .env("HOME", &self.home)
            .output()
            .unwrap()
    }

    fn write(&self, agent: &str, value: &Value) -> Vec<u8> {
        let path = self.config(agent);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let bytes = serde_json::to_vec(value).unwrap();
        fs::write(path, &bytes).unwrap();
        bytes
    }
}

fn nested(event: &str, handler: Value) -> Value {
    json!({"hooks": {event: [{"hooks": [handler]}]}})
}

fn direct(event: &str, handler: Value) -> Value {
    json!({"version": 1, "hooks": {event: [handler]}})
}

fn assert_failure_without_mutation(agent: &str, label: &str, config: Value) {
    let harness = Harness::new();
    let before = harness.write(agent, &config);
    let output = harness.install(agent);
    assert_eq!(output.status.code(), Some(2), "{label}");
    assert!(!output.stderr.is_empty(), "{label}");
    assert_eq!(fs::read(harness.config(agent)).unwrap(), before, "{label}");
}

fn assert_success(output: &Output, label: &str) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{label}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn codex_known_handler_fields_fail_closed_by_agent_event_and_variant() {
    for (label, event, handler) in [
        (
            "codex Stop command timeout",
            "Stop",
            json!({"type":"command","command":"x","timeout":"30"}),
        ),
        (
            "codex Stop command timeout negative",
            "Stop",
            json!({"type":"command","command":"x","timeout":-1}),
        ),
        (
            "codex Stop command timeout fractional",
            "Stop",
            json!({"type":"command","command":"x","timeout":1.5}),
        ),
        (
            "codex Stop command statusMessage",
            "Stop",
            json!({"type":"command","command":"x","statusMessage":false}),
        ),
        (
            "codex Stop command additionalContextLimit negative",
            "Stop",
            json!({"type":"command","command":"x","additionalContextLimit":-1}),
        ),
        (
            "codex Stop command additionalContextLimit fractional",
            "Stop",
            json!({"type":"command","command":"x","additionalContextLimit":1.5}),
        ),
        (
            "codex Stop command commandWindows",
            "Stop",
            json!({"type":"command","command":"x","commandWindows":false}),
        ),
        (
            "codex Stop command command_windows",
            "Stop",
            json!({"type":"command","command":"x","command_windows":false}),
        ),
        (
            "codex Stop command duplicate Windows aliases",
            "Stop",
            json!({
                "type":"command", "command":"x", "commandWindows":"x.exe",
                "command_windows":"x.exe"
            }),
        ),
        (
            "codex Stop command async",
            "Stop",
            json!({"type":"command","command":"x","async":"yes"}),
        ),
        (
            "codex SessionEnd command timeout range",
            "SessionEnd",
            json!({"type":"command","command":"x","timeout":4}),
        ),
    ] {
        assert_failure_without_mutation("codex", label, nested(event, handler));
    }
}

#[test]
fn claude_known_handler_fields_fail_closed_by_agent_event_and_variant() {
    for (label, event, handler) in [
        (
            "claude Stop command timeout",
            "Stop",
            json!({"type":"command","command":"x","timeout":"30"}),
        ),
        (
            "claude Stop command statusMessage",
            "Stop",
            json!({"type":"command","command":"x","statusMessage":false}),
        ),
        (
            "claude Stop command once",
            "Stop",
            json!({"type":"command","command":"x","once":"yes"}),
        ),
        (
            "claude Stop command if",
            "Stop",
            json!({"type":"command","command":"x","if":false}),
        ),
        (
            "claude Stop command args",
            "Stop",
            json!({"type":"command","command":"x","args":["ok",1]}),
        ),
        (
            "claude Stop command async",
            "Stop",
            json!({"type":"command","command":"x","async":"yes"}),
        ),
        (
            "claude Stop command asyncRewake",
            "Stop",
            json!({"type":"command","command":"x","asyncRewake":"yes"}),
        ),
        (
            "claude Stop command shell",
            "Stop",
            json!({"type":"command","command":"x","shell":"zsh"}),
        ),
        (
            "claude Stop http headers",
            "Stop",
            json!({"type":"http","url":"https://example.test","headers":{"x":1}}),
        ),
        (
            "claude Stop http allowedEnvVars",
            "Stop",
            json!({"type":"http","url":"https://example.test","allowedEnvVars":["A",1]}),
        ),
        (
            "claude Stop prompt model",
            "Stop",
            json!({"type":"prompt","prompt":"x","model":1}),
        ),
        (
            "claude Stop prompt continueOnBlock",
            "Stop",
            json!({"type":"prompt","prompt":"x","continueOnBlock":"yes"}),
        ),
        (
            "claude Stop agent continueOnBlock incompatible",
            "Stop",
            json!({"type":"agent","prompt":"x","continueOnBlock":true}),
        ),
        (
            "claude Stop command url incompatible",
            "Stop",
            json!({"type":"command","command":"x","url":"https://example.test"}),
        ),
        (
            "claude SessionEnd command timeout range",
            "SessionEnd",
            json!({"type":"command","command":"x","timeout":61}),
        ),
        (
            "claude ConfigChange prompt matrix",
            "ConfigChange",
            json!({"type":"prompt","prompt":"x"}),
        ),
    ] {
        assert_failure_without_mutation("claude-code", label, nested(event, handler));
    }
}

#[test]
fn claude_event_variant_matrix_fails_closed_and_accepts_documented_pairs() {
    for (label, event, handler) in [
        (
            "claude SessionStart prompt",
            "SessionStart",
            json!({"type":"prompt","prompt":"x"}),
        ),
        (
            "claude SessionStart http",
            "SessionStart",
            json!({"type":"http","url":"https://example.test"}),
        ),
        (
            "claude PreCompact prompt",
            "PreCompact",
            json!({"type":"prompt","prompt":"x"}),
        ),
    ] {
        assert_failure_without_mutation("claude-code", label, nested(event, handler));
    }

    for (label, event, handler) in [
        (
            "claude SessionStart command",
            "SessionStart",
            json!({"type":"command","command":"x"}),
        ),
        (
            "claude SessionStart mcp_tool",
            "SessionStart",
            json!({"type":"mcp_tool","server":"s","tool":"t"}),
        ),
        (
            "claude PreCompact http",
            "PreCompact",
            json!({"type":"http","url":"https://example.test"}),
        ),
        (
            "claude Stop agent",
            "Stop",
            json!({"type":"agent","prompt":"x"}),
        ),
    ] {
        let harness = Harness::new();
        harness.write("claude-code", &nested(event, handler));
        assert_success(&harness.install("claude-code"), label);
    }
}

#[test]
fn cursor_known_handler_fields_fail_closed_by_agent_event_and_variant() {
    for (label, event, handler) in [
        (
            "cursor stop command timeout",
            "stop",
            json!({"command":"x","timeout":"30"}),
        ),
        (
            "cursor stop command failClosed",
            "stop",
            json!({"command":"x","failClosed":"yes"}),
        ),
        (
            "cursor stop command loop_limit",
            "stop",
            json!({"command":"x","loop_limit":false}),
        ),
        (
            "cursor stop command matcher",
            "stop",
            json!({"command":"x","matcher":[]}),
        ),
        (
            "cursor stop prompt model",
            "stop",
            json!({"type":"prompt","prompt":"x","model":1}),
        ),
        (
            "cursor stop prompt command incompatible",
            "stop",
            json!({"type":"prompt","prompt":"x","command":"x"}),
        ),
        (
            "cursor stop command prompt incompatible",
            "stop",
            json!({"type":"command","command":"x","prompt":"x"}),
        ),
        (
            "cursor preToolUse command loop_limit incompatible",
            "preToolUse",
            json!({"command":"x","loop_limit":5}),
        ),
        (
            "cursor beforeReadFile command failClosed",
            "beforeReadFile",
            json!({"command":"x","failClosed":"yes"}),
        ),
    ] {
        assert_failure_without_mutation("cursor", label, direct(event, handler));
    }
}

#[test]
fn documented_fields_and_unknown_extensions_are_preserved() {
    let harness = Harness::new();
    harness.write(
        "codex",
        &json!({"hooks":{"SessionEnd":[{"hooks":[
            {
                "type":"command", "command":"third-party", "timeout":3,
                "statusMessage":"done", "additionalContextLimit":0,
                "commandWindows":"third-party.exe", "async":false, "futureField":{"x":1}
            },
            {"type":"command", "command":"other", "command_windows":"other.exe"}
        ]}]}}),
    );
    assert_success(&harness.install("codex"), "codex SessionEnd command valid");
    let codex: Value = serde_json::from_slice(&fs::read(harness.config("codex")).unwrap()).unwrap();
    assert_eq!(
        codex["hooks"]["SessionEnd"][0]["hooks"][0]["futureField"]["x"],
        1
    );
    assert_eq!(
        codex["hooks"]["SessionEnd"][0]["hooks"][1]["command_windows"],
        "other.exe"
    );

    let harness = Harness::new();
    harness.write(
        "claude-code",
        &json!({"hooks":{"Stop":[{"hooks":[
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
        ]}]}}),
    );
    assert_success(
        &harness.install("claude-code"),
        "claude Stop all variants valid",
    );
    let claude: Value =
        serde_json::from_slice(&fs::read(harness.config("claude-code")).unwrap()).unwrap();
    assert_eq!(
        claude["hooks"]["Stop"][0]["hooks"][0]["futureField"]["x"],
        1
    );
    assert_eq!(
        claude["hooks"]["Stop"][0]["hooks"]
            .as_array()
            .unwrap()
            .len(),
        5
    );

    let harness = Harness::new();
    harness.write(
        "cursor",
        &json!({"version":1.5,"hooks":{"stop":[
            {
                "command":"third-party", "timeout":1, "loop_limit":null,
                "failClosed":true, "matcher":{"type":"Shell"}, "futureField":{"x":1}
            },
            {
                "type":"prompt", "prompt":"review", "model":"fast", "timeout":1,
                "loop_limit":5, "failClosed":false, "matcher":"Stop"
            }
        ]}}),
    );
    assert_success(
        &harness.install("cursor"),
        "cursor stop command valid object matcher",
    );
    let cursor: Value =
        serde_json::from_slice(&fs::read(harness.config("cursor")).unwrap()).unwrap();
    assert_eq!(cursor["hooks"]["stop"][0]["futureField"]["x"], 1);
    assert_eq!(cursor["version"], 1.5);
}

#[test]
fn future_event_and_variant_extensions_remain_opaque() {
    for (agent, event, value) in [
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
    ] {
        let harness = Harness::new();
        harness.write(agent, &value);
        assert_success(&harness.install(agent), agent);
        let installed: Value =
            serde_json::from_slice(&fs::read(harness.config(agent)).unwrap()).unwrap();
        let handler = if agent == "cursor" {
            &installed["hooks"][event][0]
        } else {
            &installed["hooks"][event][0]["hooks"][0]
        };
        assert_eq!(handler["timeout"], "future");
        assert_eq!(handler["future"], true);
    }
}

#[test]
fn cursor_accepts_both_documented_matcher_shapes() {
    for (label, matcher) in [
        (
            "cursor preToolUse command string matcher",
            json!("Shell|Read"),
        ),
        (
            "cursor preToolUse command object matcher",
            json!({"type":"Shell"}),
        ),
    ] {
        let harness = Harness::new();
        harness.write(
            "cursor",
            &direct("preToolUse", json!({"command":"x","matcher":matcher})),
        );
        assert_success(&harness.install("cursor"), label);
    }
}
