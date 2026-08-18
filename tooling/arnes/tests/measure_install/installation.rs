use super::*;

const CODEX_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PostToolUse",
    "PreCompact",
    "PostCompact",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "SessionEnd",
];
const CLAUDE_EVENTS: &[&str] = &[
    "SessionStart",
    "UserPromptSubmit",
    "PreToolUse",
    "PermissionRequest",
    "PermissionDenied",
    "PostToolUse",
    "PostToolUseFailure",
    "SubagentStart",
    "SubagentStop",
    "Stop",
    "StopFailure",
    "PreCompact",
    "PostCompact",
    "SessionEnd",
];
const CURSOR_EVENTS: &[&str] = &[
    "sessionStart",
    "beforeSubmitPrompt",
    "preToolUse",
    "postToolUse",
    "postToolUseFailure",
    "subagentStart",
    "subagentStop",
    "afterAgentResponse",
    "stop",
    "preCompact",
    "postCompact",
    "sessionEnd",
];

#[test]
fn installs_hooks_when_agent_configuration_is_absent() {
    for agent in ["codex", "claude-code", "cursor"] {
        assert_hooks_installed(agent);
    }
}

fn assert_hooks_installed(agent: &str) {
    let harness = Harness::new();
    assert_success(&harness.install(agent));
    let config = read_json(harness.config(agent));
    let hooks = config["hooks"].as_object().unwrap();
    let mut actual: Vec<&str> = hooks.keys().map(String::as_str).collect();
    actual.sort_unstable();
    let mut expected = expected_events(agent).to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected);
    assert!(!hooks.contains_key("afterAgentThought"));
    for entries in hooks.values() {
        let command = match agent {
            "cursor" => &entries[0]["command"],
            _ => &entries[0]["hooks"][0]["command"],
        };
        assert_eq!(command.as_str().unwrap(), harness.command(agent));
    }
    match agent {
        "cursor" => assert_eq!(config["version"], 1),
        _ => assert!(config.get("version").is_none()),
    }
}

fn expected_events(agent: &str) -> &'static [&'static str] {
    match agent {
        "codex" => CODEX_EVENTS,
        "claude-code" => CLAUDE_EVENTS,
        "cursor" => CURSOR_EVENTS,
        _ => unreachable!(),
    }
}

#[test]
fn preserves_third_party_hooks_matchers_top_level_settings_and_cursor_version() {
    for agent in ["codex", "claude-code"] {
        assert_nested_third_party_config_preserved(agent);
    }
    assert_cursor_third_party_config_preserved();
}

fn assert_nested_third_party_config_preserved(agent: &str) {
    let harness = Harness::new();
    let command = harness.command(agent);
    harness.write_config(
        agent,
        &serde_json::json!({
            "theme":"dark",
            "hooks":{
                "Stop":[
                    {"matcher":"empty", "hooks":[]},
                    {"matcher":"kept", "hooks":[{"type":"command","command":"third-party"}]}
                ],
                "FutureHooks":[{"hooks":[
                    {"type":"command","command":command.clone()},
                    {"type":"prompt","prompt":"keep","futureField":command}
                ]}],
                "FutureEvent":{"opaque":true}
            }
        }),
    );
    assert_success(&harness.install(agent));
    let config = read_json(harness.config(agent));
    assert_eq!(config["theme"], "dark");
    assert_eq!(config["hooks"]["FutureEvent"]["opaque"], true);
    assert_eq!(
        config["hooks"]["FutureHooks"][0]["hooks"][0]["type"],
        "prompt"
    );
    assert_eq!(
        config["hooks"]["FutureHooks"][0]["hooks"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        config["hooks"]["Stop"][0],
        serde_json::json!({"matcher":"empty","hooks":[]})
    );
    assert_eq!(config["hooks"]["Stop"][1]["matcher"], "kept");
    assert_eq!(
        config["hooks"]["Stop"][1]["hooks"][0]["command"],
        "third-party"
    );
}

fn assert_cursor_third_party_config_preserved() {
    let harness = Harness::new();
    harness.write_config(
        "cursor",
        &serde_json::json!({
            "version":7, "theme":"dark", "hooks":{
                "stop":[{"command":"third-party","matcher":"kept"}],
                "futureEvent":{"opaque":true}
            }
        }),
    );
    assert_success(&harness.install("cursor"));
    let config = read_json(harness.config("cursor"));
    assert_eq!(config["version"], 7);
    assert_eq!(config["theme"], "dark");
    assert_eq!(config["hooks"]["futureEvent"]["opaque"], true);
    assert_eq!(config["hooks"]["stop"][0]["command"], "third-party");
    assert_eq!(config["hooks"]["stop"][0]["matcher"], "kept");
}

#[test]
fn installation_is_byte_idempotent_and_collapses_exact_duplicates() {
    for agent in ["codex", "claude-code", "cursor"] {
        let harness = Harness::new();
        let command = harness.command(agent);
        let duplicate = if agent == "cursor" {
            serde_json::json!({"version":1,"hooks":{"stop":[
                {"command":command,"old":true},
                {"command":"third-party"},
                {"command":command,"other":true}
            ]}})
        } else {
            serde_json::json!({"hooks":{"Stop":[
                {"matcher":"old","hooks":[{"type":"command","command":command}]},
                {"hooks":[{"type":"command","command":"third-party"},{"type":"command","command":command}]}
            ]}})
        };
        harness.write_config(agent, &duplicate);

        assert_success(&harness.install(agent));
        let once = fs::read(harness.config(agent)).unwrap();
        assert_success(&harness.install(agent));
        let twice = fs::read(harness.config(agent)).unwrap();

        assert_eq!(once, twice);
        let config: Value = serde_json::from_slice(&once).unwrap();
        let serialized = serde_json::to_string(&config).unwrap();
        let expected = match agent {
            "codex" => 11,
            "claude-code" => 14,
            "cursor" => 12,
            _ => unreachable!(),
        };
        assert_eq!(serialized.matches(&command).count(), expected);
        assert!(serialized.contains("third-party"));
    }
}

#[test]
fn quotes_spaces_and_single_quotes_in_the_executable_path() {
    let harness = Harness::with_command_name("ar nes's");
    assert_success(&harness.install("codex"));
    let config = read_json(harness.config("codex"));
    let command = config["hooks"]["Stop"][0]["hooks"][0]["command"]
        .as_str()
        .unwrap();
    assert_eq!(command, harness.command("codex"));
    assert!(command.contains("'\\''"));
}

#[test]
fn rejects_relative_missing_and_non_file_commands_without_creating_configuration() {
    let harness = Harness::new();
    let non_executable = harness.home.join("non-executable");
    fs::write(&non_executable, b"binary").unwrap();
    fs::set_permissions(&non_executable, fs::Permissions::from_mode(0o600)).unwrap();
    for command in [
        PathBuf::from("relative/arnes"),
        harness.home.join("missing"),
        harness.home.clone(),
        non_executable,
    ] {
        assert_failure(&harness.install_with("codex", &command));
        assert!(!harness.config("codex").exists());
    }
}

#[test]
fn accepts_an_executable_symlink_like_the_deployed_arnes_command() {
    let harness = Harness::new();
    let command = harness.home.join("arnes-link");
    symlink(&harness.executable, &command).unwrap();
    assert_success(&harness.install_with("codex", &command));
    let config = read_json(harness.config("codex"));
    assert_eq!(
        config["hooks"]["Stop"][0]["hooks"][0]["command"],
        format!("'{}' measure hook --agent codex", command.display())
    );
}
