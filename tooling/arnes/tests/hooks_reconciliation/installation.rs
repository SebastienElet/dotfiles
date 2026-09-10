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
fn installs_hooks_when_agent_configuration_is_absent()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for agent in ["codex", "claude-code", "cursor"] {
        assert_hooks_installed(agent)?;
    };
    Ok(())
}
fn assert_hooks_installed(agent: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_success(&harness.install(agent)?);
    let config = read_json(harness.config(agent)?)?;
    let hooks = (*(config).get("hooks").ok_or("missing fixture index hooks")?)
        .as_object()
        .ok_or("expected JSON object")?;
    let mut actual: Vec<&str> = hooks.keys().map(String::as_str).collect();
    actual.sort_unstable();
    let mut expected = expected_events(agent)?.to_vec();
    expected.sort_unstable();
    assert_eq!(actual, expected);
    assert!(!hooks.contains_key("afterAgentThought"));
    for entries in hooks.values() {
        let command = match agent {
            "cursor" => (*(entries).get(0).ok_or("missing fixture index 0")?)
                .get("command")
                .ok_or("missing fixture index command")?,
            _ => (*(*(*(entries).get(0).ok_or("missing fixture index 0")?)
                .get("hooks")
                .ok_or("missing fixture index hooks")?)
            .get(0)
            .ok_or("missing fixture index 0")?)
            .get("command")
            .ok_or("missing fixture index command")?,
        };
        assert_eq!(
            command.as_str().ok_or("expected JSON string")?,
            harness.command(agent)?
        );
    }
    let _: () = match agent {
        "cursor" => assert_eq!(
            *(config)
                .get("version")
                .ok_or("missing fixture index version")?,
            1
        ),
        _ => assert!(config.get("version").is_none()),
    };
    Ok(())
}
fn expected_events(
    agent: &str,
) -> Result<&'static [&'static str], Box<dyn std::error::Error + Send + Sync>> {
    match agent {
        "codex" => Ok(CODEX_EVENTS),
        "claude-code" => Ok(CLAUDE_EVENTS),
        "cursor" => Ok(CURSOR_EVENTS),
        _ => Err(format!("unsupported test agent: {agent}").into()),
    }
}
#[test]
fn preserves_third_party_hooks_matchers_top_level_settings_and_cursor_version()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for agent in ["codex", "claude-code"] {
        assert_nested_third_party_config_preserved(agent)?;
    }
    assert_cursor_third_party_config_preserved()?;
    Ok(())
}
fn assert_nested_third_party_config_preserved(
    agent: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let command = harness.command(agent)?;
    harness . write_config (agent , & serde_json :: json ! ({ "theme" : "dark" , "hooks" : { "Stop" : [{ "matcher" : "empty" , "hooks" : [] } , { "matcher" : "kept" , "hooks" : [{ "type" : "command" , "command" : "third-party" }] }] , "FutureHooks" : [{ "hooks" : [{ "type" : "command" , "command" : command } , { "type" : "prompt" , "prompt" : "keep" , "futureField" : command }] }] , "FutureEvent" : { "opaque" : true } } }) ,) ? ;
    assert_success(&harness.install(agent)?);
    let config = read_json(harness.config(agent)?)?;
    assert_eq!(
        *(config).get("theme").ok_or("missing fixture index theme")?,
        "dark"
    );
    assert_eq!(
        *(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("FutureEvent")
            .ok_or("missing fixture index FutureEvent")?)
        .get("opaque")
        .ok_or("missing fixture index opaque")?,
        true
    );
    assert_eq!(
        *(*(*(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("FutureHooks")
            .ok_or("missing fixture index FutureHooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("type")
        .ok_or("missing fixture index type")?,
        "prompt"
    );
    assert_eq!(
        (*(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("FutureHooks")
            .ok_or("missing fixture index FutureHooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len(),
        1
    );
    assert_eq!(
        *(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("Stop")
            .ok_or("missing fixture index Stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?,
        serde_json :: json ! ({ "matcher" : "empty" , "hooks" : [] })
    );
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("Stop")
            .ok_or("missing fixture index Stop")?)
        .get(1)
        .ok_or("missing fixture index 1")?)
        .get("matcher")
        .ok_or("missing fixture index matcher")?,
        "kept"
    );
    assert_eq!(
        *(*(*(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("Stop")
            .ok_or("missing fixture index Stop")?)
        .get(1)
        .ok_or("missing fixture index 1")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        "third-party"
    );
    Ok(())
}
fn assert_cursor_third_party_config_preserved()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness . write_config ("cursor" , & serde_json :: json ! ({ "version" : 7 , "theme" : "dark" , "hooks" : { "stop" : [{ "command" : "third-party" , "matcher" : "kept" }] , "futureEvent" : { "opaque" : true } } }) ,) ? ;
    assert_success(&harness.install("cursor")?);
    let config = read_json(harness.config("cursor")?)?;
    assert_eq!(
        *(config)
            .get("version")
            .ok_or("missing fixture index version")?,
        7
    );
    assert_eq!(
        *(config).get("theme").ok_or("missing fixture index theme")?,
        "dark"
    );
    assert_eq!(
        *(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("futureEvent")
            .ok_or("missing fixture index futureEvent")?)
        .get("opaque")
        .ok_or("missing fixture index opaque")?,
        true
    );
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("stop")
            .ok_or("missing fixture index stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        "third-party"
    );
    assert_eq!(
        *(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("stop")
            .ok_or("missing fixture index stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("matcher")
        .ok_or("missing fixture index matcher")?,
        "kept"
    );
    Ok(())
}
#[test]
fn installation_is_byte_idempotent_and_collapses_exact_duplicates()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for agent in ["codex", "claude-code", "cursor"] {
        let harness = Harness::new()?;
        let command = harness.command(agent)?;
        let duplicate = if agent == "cursor" {
            serde_json :: json ! ({ "version" : 1 , "hooks" : { "stop" : [{ "command" : command , "old" : true } , { "command" : "third-party" } , { "command" : command , "other" : true }] } })
        } else {
            serde_json :: json ! ({ "hooks" : { "Stop" : [{ "matcher" : "old" , "hooks" : [{ "type" : "command" , "command" : command }] } , { "hooks" : [{ "type" : "command" , "command" : "third-party" } , { "type" : "command" , "command" : command }] }] } })
        };
        harness.write_config(agent, &duplicate)?;
        assert_success(&harness.install(agent)?);
        let once = fs::read(harness.config(agent)?)?;
        assert_success(&harness.install(agent)?);
        let twice = fs::read(harness.config(agent)?)?;
        assert_eq!(once, twice);
        let config: Value = serde_json::from_slice(&once)?;
        let serialized = serde_json::to_string(&config)?;
        let expected = match agent {
            "codex" => 11,
            "claude-code" => 14,
            "cursor" => 12,
            _ => return Err(format!("unsupported test agent: {agent}").into()),
        };
        assert_eq!(serialized.matches(&command).count(), expected);
        assert!(serialized.contains("third-party"));
    };
    Ok(())
}
#[test]
fn quotes_spaces_and_single_quotes_in_the_executable_path()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::with_command_name("ar nes's")?;
    assert_success(&harness.install("codex")?);
    let config = read_json(harness.config("codex")?)?;
    let command = (*(*(*(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
        .get("Stop")
        .ok_or("missing fixture index Stop")?)
    .get(0)
    .ok_or("missing fixture index 0")?)
    .get("hooks")
    .ok_or("missing fixture index hooks")?)
    .get(0)
    .ok_or("missing fixture index 0")?)
    .get("command")
    .ok_or("missing fixture index command")?)
    .as_str()
    .ok_or("expected JSON string")?;
    assert_eq!(command, harness.command("codex")?);
    assert!(command.contains("'\\''"));
    Ok(())
}
#[test]
fn rejects_unavailable_and_non_executable_managed_commands_without_creating_configuration()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let non_executable = harness.home.join("non-executable");
    fs::write(&non_executable, b"binary")?;
    fs::set_permissions(&non_executable, fs::Permissions::from_mode(0o600))?;
    let _: () = for command in [
        PathBuf::from("relative/arnes"),
        harness.home.join("missing"),
        harness.home.clone(),
        non_executable,
    ] {
        assert_failure(&harness.install_with("codex", &command)?);
        assert!(!harness.config("codex")?.exists());
    };
    Ok(())
}
#[test]
fn accepts_an_executable_symlink_like_the_deployed_arnes_command()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let target = harness.home.join("arnes-target");
    fs::write(&target, b"binary")?;
    fs::set_permissions(&target, fs::Permissions::from_mode(0o700))?;
    fs::remove_file(&harness.executable)?;
    symlink(&target, &harness.executable)?;
    assert_success(&harness.install("codex")?);
    let config = read_json(harness.config("codex")?)?;
    assert_eq!(
        *(*(*(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("Stop")
            .ok_or("missing fixture index Stop")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("command")
        .ok_or("missing fixture index command")?,
        format!(
            "'{}' measure hook --agent codex",
            harness.executable.display()
        )
    );
    Ok(())
}
#[test]
fn installs_memory_hook_on_user_prompt_submit_only()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for agent in ["codex", "claude-code"] {
        let harness = Harness::new()?;
        harness.write_memory_manifest(agent)?;
        harness.executable_named("agent-memory")?;
        assert_success(&harness.install(agent)?);
        let config = read_json(harness.config(agent)?)?;
        let hooks = (*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .as_object()
            .ok_or("expected JSON object")?;
        assert_eq!(
            hooks.keys().collect::<Vec<&String>>(),
            vec!["UserPromptSubmit"]
        );
        let handler = (*(*(*(hooks)
            .get("UserPromptSubmit")
            .ok_or("missing fixture index UserPromptSubmit")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?;
        assert_eq!(
            *(handler).get("type").ok_or("missing fixture index type")?,
            "command"
        );
        assert_eq!(
            *(handler)
                .get("command")
                .ok_or("missing fixture index command")?,
            harness.memory_command(agent)?
        );
        assert_eq!(
            *(handler)
                .get("timeout")
                .ok_or("missing fixture index timeout")?,
            30
        );
    };
    Ok(())
}
#[test]
fn concurrent_memory_installations_leave_one_owned_hook_and_visible_failures()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_memory_manifest("codex")?;
    harness.executable_named("agent-memory")?;
    let first = spawn_captured(harness.setup_command("codex"))?;
    let second = spawn_captured(harness.setup_command("codex"))?;
    let first = first.wait_with_output()?;
    let second = second.wait_with_output()?;
    let outputs = [&first, &second];
    assert!(
        outputs.iter().any(|output| output.status.code() == Some(0)),
        "neither concurrent installation succeeded"
    );
    for output in outputs {
        if output.status.code() != Some(0) {
            assert_eq!(output.status.code(), Some(2));
            assert!(!output.stderr.is_empty());
        }
    }
    let concurrent = fs::read(harness.config("codex")?)?;
    let config: Value = serde_json::from_slice(&concurrent)?;
    let serialized = serde_json::to_string(&config)?;
    assert_eq!(
        serialized
            .matches(&harness.memory_command("codex")?)
            .count(),
        1
    );
    assert_eq!(
        *(*(*(*(*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .get("UserPromptSubmit")
            .ok_or("missing fixture index UserPromptSubmit")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("timeout")
        .ok_or("missing fixture index timeout")?,
        30
    );
    assert_success(&harness.install("codex")?);
    assert_eq!(fs::read(harness.config("codex")?)?, concurrent);
    Ok(())
}
fn spawn_captured(
    mut command: Command,
) -> Result<std::process::Child, Box<dyn std::error::Error + Send + Sync>> {
    Ok(command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?)
}
