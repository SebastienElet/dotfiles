use serde_json::Value;
use std::fs;
use std::os::unix::fs::{FileTypeExt, PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
    executable: PathBuf,
}

impl Harness {
    fn new() -> Self {
        Self::with_command_name("arnes")
    }

    fn with_command_name(name: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let executable = root.path().join("bin").join(name);
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

    fn install(&self, agent: &str) -> Output {
        self.install_with(agent, &self.executable)
    }

    fn install_with(&self, agent: &str, command_path: &Path) -> Output {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["measure", "install-hooks", "--agent", agent, "--command"])
            .arg(command_path)
            .env_clear()
            .env("HOME", &self.home)
            .output()
            .unwrap()
    }

    fn config(&self, agent: &str) -> PathBuf {
        let relative = match agent {
            "codex" => ".codex/hooks.json",
            "claude-code" => ".claude/settings.json",
            "cursor" => ".cursor/hooks.json",
            _ => unreachable!(),
        };
        self.home.join(relative)
    }

    fn command(&self, agent: &str) -> String {
        let path = self.executable.to_str().unwrap().replace('\'', "'\\''");
        format!("'{path}' measure hook --agent {agent}")
    }

    fn write_config(&self, agent: &str, value: &Value) {
        let path = self.config(agent);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    }
}

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

fn assert_failure(output: &Output) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

#[test]
fn installs_hooks_when_agent_configuration_is_absent() {
    for (agent, expected) in [
        (
            "codex",
            &[
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
            ][..],
        ),
        (
            "claude-code",
            &[
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
            ][..],
        ),
        (
            "cursor",
            &[
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
                "sessionEnd",
            ][..],
        ),
    ] {
        let harness = Harness::new();

        assert_success(&harness.install(agent));

        let config = read_json(harness.config(agent));
        let hooks = config["hooks"].as_object().unwrap();
        let mut actual: Vec<&str> = hooks.keys().map(String::as_str).collect();
        actual.sort_unstable();
        let mut expected = expected.to_vec();
        expected.sort_unstable();
        assert_eq!(actual, expected);
        assert!(!hooks.contains_key("afterAgentThought"));
        for entries in hooks.values() {
            let command = if agent == "cursor" {
                entries[0]["command"].as_str().unwrap()
            } else {
                entries[0]["hooks"][0]["command"].as_str().unwrap()
            };
            assert_eq!(command, harness.command(agent));
        }
        if agent == "cursor" {
            assert_eq!(config["version"], 1);
        } else {
            assert!(config.get("version").is_none());
        }
    }
}

#[test]
fn preserves_third_party_hooks_matchers_top_level_settings_and_cursor_version() {
    for agent in ["codex", "claude-code"] {
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
        assert_eq!(config["hooks"]["Stop"][0]["matcher"], "empty");
        assert_eq!(config["hooks"]["Stop"][0]["hooks"], serde_json::json!([]));
        assert_eq!(config["hooks"]["Stop"][1]["matcher"], "kept");
        assert_eq!(
            config["hooks"]["Stop"][1]["hooks"][0]["command"],
            "third-party"
        );
    }

    let harness = Harness::new();
    harness.write_config(
        "cursor",
        &serde_json::json!({
            "version":7,
            "theme":"dark",
            "hooks":{
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
            "cursor" => 11,
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
    let harness = Harness::new();
    harness.write_config(
        "codex",
        &serde_json::json!({"hooks":{"Stop":[{"hooks":[
            {"type":"prompt","prompt":"review"},
            {"type":"agent","prompt":"investigate"}
        ]}]}}),
    );
    assert_success(&harness.install("codex"));
    let codex = read_json(harness.config("codex"));
    assert_eq!(
        codex["hooks"]["Stop"][0]["hooks"].as_array().unwrap().len(),
        2
    );

    let harness = Harness::new();
    harness.write_config(
        "claude-code",
        &serde_json::json!({"hooks":{"Stop":[{"hooks":[
            {"type":"command","command":"third-party"},
            {"type":"http","url":"https://example.com/hook"},
            {"type":"mcp_tool","server":"review","tool":"record","input":{"ok":true}},
            {"type":"prompt","prompt":"review"},
            {"type":"agent","prompt":"investigate"}
        ]}]}}),
    );
    assert_success(&harness.install("claude-code"));
    let claude = read_json(harness.config("claude-code"));
    assert_eq!(
        claude["hooks"]["Stop"][0]["hooks"]
            .as_array()
            .unwrap()
            .len(),
        5
    );

    let harness = Harness::new();
    harness.write_config(
        "cursor",
        &serde_json::json!({"version":1,"hooks":{"stop":[
            {"command":"third-party"},
            {"type":"command","command":"typed-third-party"},
            {"type":"prompt","prompt":"review"}
        ]}}),
    );
    assert_success(&harness.install("cursor"));
    let cursor = read_json(harness.config("cursor"));
    assert_eq!(cursor["hooks"]["stop"].as_array().unwrap().len(), 4);
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

#[test]
fn symlink_directory_and_unreadable_configurations_are_rejected_without_mutation() {
    let harness = Harness::new();
    let config = harness.config("codex");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let victim = harness.home.join("victim");
    fs::write(&victim, br#"{"hooks":{}}"#).unwrap();
    symlink(&victim, &config).unwrap();

    assert_failure(&harness.install("codex"));
    assert_eq!(fs::read(&victim).unwrap(), br#"{"hooks":{}}"#);
    assert!(
        fs::symlink_metadata(&config)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    fs::remove_file(&config).unwrap();
    fs::create_dir(&config).unwrap();
    assert_failure(&harness.install("codex"));
    assert!(config.is_dir());

    fs::remove_dir(&config).unwrap();
    fs::write(&config, br#"{"hooks":{}}"#).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
    assert_failure(&harness.install("codex"));
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read(&config).unwrap(), br#"{"hooks":{}}"#);
}

#[test]
fn fifo_hardlink_and_symlinked_agent_directory_are_rejected() {
    let harness = Harness::new();
    let config = harness.config("codex");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(&config)
            .status()
            .unwrap()
            .success()
    );
    assert_failure(&harness.install("codex"));
    assert!(fs::symlink_metadata(&config).unwrap().file_type().is_fifo());

    fs::remove_file(&config).unwrap();
    let other = harness.home.join("other");
    fs::write(&other, br#"{"hooks":{}}"#).unwrap();
    fs::hard_link(&other, &config).unwrap();
    assert_failure(&harness.install("codex"));
    assert_eq!(fs::read(&other).unwrap(), br#"{"hooks":{}}"#);

    fs::remove_file(&config).unwrap();
    fs::remove_file(config.parent().unwrap().join(".hooks.json.lock")).unwrap();
    fs::remove_dir(config.parent().unwrap()).unwrap();
    let actual = harness.home.join("actual-codex");
    fs::create_dir(&actual).unwrap();
    symlink(&actual, config.parent().unwrap()).unwrap();
    assert_failure(&harness.install("codex"));
    assert!(!actual.join("hooks.json").exists());
}

#[test]
fn predictable_temporary_and_lock_symlinks_are_never_followed() {
    let harness = Harness::new();
    let config = harness.config("codex");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let victim = harness.home.join("victim");
    fs::write(&victim, b"keep").unwrap();
    let predictable = config.parent().unwrap().join("hooks.json.tmp");
    symlink(&victim, &predictable).unwrap();

    assert_success(&harness.install("codex"));
    assert_eq!(fs::read(&victim).unwrap(), b"keep");
    assert!(
        fs::symlink_metadata(&predictable)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    fs::remove_file(config.parent().unwrap().join(".hooks.json.lock")).unwrap();
    symlink(&victim, config.parent().unwrap().join(".hooks.json.lock")).unwrap();
    let before = fs::read(&config).unwrap();
    assert_failure(&harness.install("codex"));
    assert_eq!(fs::read(config).unwrap(), before);
    assert_eq!(fs::read(victim).unwrap(), b"keep");
}

#[test]
fn creates_private_configuration_and_preserves_an_existing_mode() {
    let harness = Harness::new();
    assert_success(&harness.install("codex"));
    let config = harness.config("codex");
    assert_eq!(
        fs::metadata(&config).unwrap().permissions().mode() & 0o777,
        0o600
    );

    fs::set_permissions(&config, fs::Permissions::from_mode(0o640)).unwrap();
    let value = read_json(&config);
    let mut compact = serde_json::to_vec(&value).unwrap();
    compact.push(b' ');
    fs::write(&config, compact).unwrap();
    assert_success(&harness.install("codex"));
    assert_eq!(
        fs::metadata(config).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

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
fn unwritable_configuration_directory_is_rejected_without_mutation() {
    let harness = Harness::new();
    let config = harness.config("codex");
    let directory = config.parent().unwrap();
    fs::create_dir_all(directory).unwrap();
    fs::write(&config, br#"{"hooks":{}}"#).unwrap();
    fs::set_permissions(directory, fs::Permissions::from_mode(0o500)).unwrap();

    let output = harness.install("codex");

    fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
    assert_failure(&output);
    assert_eq!(fs::read(&config).unwrap(), br#"{"hooks":{}}"#);
}
