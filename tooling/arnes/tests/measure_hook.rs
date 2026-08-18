use serde_json::{Value, json};
use std::fs;
use std::io::Write;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Output, Stdio};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
    repository: PathBuf,
    state: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repository = root.path().join("repository");
        let state = root.path().join("state");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&repository).unwrap();
        fs::create_dir(&state).unwrap();
        Self {
            _root: root,
            home,
            repository,
            state,
        }
    }

    fn run(&self, agent: &str, payload: &[u8]) -> Output {
        let mut child = self.command(agent).spawn().unwrap();
        child.stdin.take().unwrap().write_all(payload).unwrap();
        child.wait_with_output().unwrap()
    }

    fn command(&self, agent: &str) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arnes"));
        command
            .args(["measure", "hook", "--agent", agent])
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }

    fn measure_root(&self) -> PathBuf {
        self.state.join("dotfiles/agent-harness")
    }

    fn runs(&self) -> Vec<PathBuf> {
        let root = self.measure_root().join("runs");
        if !root.exists() {
            return Vec::new();
        }
        fs::read_dir(root)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .collect()
    }

    fn only_run(&self) -> PathBuf {
        let runs = self.runs();
        assert_eq!(runs.len(), 1, "expected one run, found {runs:?}");
        runs[0].clone()
    }
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

fn read_json(path: impl AsRef<Path>) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

fn read_jsonl(path: impl AsRef<Path>) -> Vec<Value> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect()
}

#[test]
fn accepts_exact_agent_names_and_native_session_keys() {
    for (agent, payload, session) in [
        (
            "codex",
            json!({"session_id":"codex-session","event":"SessionStart"}),
            "codex-session",
        ),
        (
            "claude-code",
            json!({"session_id":"claude-session","hook_event_name":"SessionStart"}),
            "claude-session",
        ),
        (
            "cursor",
            json!({"conversation_id":"cursor-session","hook_event_name":"sessionStart"}),
            "cursor-session",
        ),
    ] {
        let harness = Harness::new();
        assert_success(&harness.run(agent, payload.to_string().as_bytes()));
        let run = read_json(harness.only_run().join("run.json"));
        assert_eq!(run["schema_version"], 1);
        assert_eq!(run["agent"], agent);
        assert_eq!(run["session_id"], session);
        assert_eq!(run["run_id"].as_str().unwrap().len(), 64);
        assert!(run["started_at_ms"].as_u64().unwrap() > 0);
        assert!(run["model"].is_null());
        assert_eq!(run["harness_fingerprint"].as_str().unwrap().len(), 64);
    }
}

#[test]
fn rejects_every_other_agent_name() {
    let harness = Harness::new();
    for agent in ["claude", "Claude-Code", "cursor-agent", "codex "] {
        let output = harness.run(agent, br#"{"session_id":"session"}"#);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.contains("invalid value"), "{stderr}");
    }
}

#[test]
fn appends_first_and_followup_prompts_with_native_ids() {
    let harness = Harness::new();
    let first = json!({
        "session_id":"session",
        "hook_event_name":"UserPromptSubmit",
        "prompt":"first prompt",
        "message_id":"message-one"
    });
    let second = json!({
        "session_id":"session",
        "hook_event_name":"UserPromptSubmit",
        "prompt":"followup prompt",
        "message_id":"message-two"
    });
    assert_success(&harness.run("claude-code", first.to_string().as_bytes()));
    assert_success(&harness.run("claude-code", second.to_string().as_bytes()));

    let prompts = read_jsonl(harness.only_run().join("prompts.jsonl"));
    assert_eq!(prompts.len(), 2);
    assert_eq!(prompts[0]["prompt"], "first prompt");
    assert_eq!(prompts[0]["prompt_id"], "message-one");
    assert_eq!(prompts[1]["prompt"], "followup prompt");
    assert_eq!(prompts[1]["prompt_id"], "message-two");
}

#[test]
fn preserves_unknown_events_and_fields_in_the_redacted_artifact() {
    let harness = Harness::new();
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"FutureEvent",
        "future":{"answer":42},
        "event_id":"native-event"
    });
    assert_success(&harness.run("codex", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let events = read_jsonl(run.join("events.jsonl"));
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["event"], "unknown");
    assert_eq!(events[0]["native_event"], "FutureEvent");
    assert_eq!(events[0]["native_ids"]["event_id"], "native-event");
    let artifact = read_json(run.join(events[0]["artifact"].as_str().unwrap()));
    assert_eq!(artifact["future"]["answer"], 42);
}

#[test]
fn normalizes_cross_agent_event_names_and_preserves_native_names() {
    for (agent, session_key, native_events) in [
        (
            "codex",
            "session_id",
            ["SessionStart", "UserPromptSubmit", "Stop", "SubagentStop"],
        ),
        (
            "claude-code",
            "session_id",
            ["SessionStart", "UserPromptSubmit", "Stop", "SubagentStop"],
        ),
        (
            "cursor",
            "conversation_id",
            ["sessionStart", "beforeSubmitPrompt", "stop", "subagentStop"],
        ),
    ] {
        let harness = Harness::new();
        for native_event in native_events {
            let mut payload = json!({"hook_event_name":native_event});
            payload[session_key] = json!("session");
            assert_success(&harness.run(agent, payload.to_string().as_bytes()));
        }
        let events = read_jsonl(harness.only_run().join("events.jsonl"));
        let normalized: Vec<&str> = events
            .iter()
            .map(|event| event["event"].as_str().unwrap())
            .collect();
        let native: Vec<&str> = events
            .iter()
            .map(|event| event["native_event"].as_str().unwrap())
            .collect();
        assert_eq!(
            normalized,
            [
                "session.start",
                "prompt.submit",
                "agent.stop",
                "subagent.stop"
            ]
        );
        assert_eq!(native, native_events);
    }
}

#[test]
fn rejects_missing_session_without_persisting_the_payload() {
    let harness = Harness::new();
    let secret = "missing-session-secret";
    let output = harness.run(
        "codex",
        json!({"event":"SessionStart","value":secret})
            .to_string()
            .as_bytes(),
    );

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("session_id")
    );
    assert!(harness.runs().is_empty());
    let invalid = fs::read_to_string(harness.measure_root().join("invalid.jsonl")).unwrap();
    assert!(!invalid.contains(secret));
}

#[test]
fn invalid_and_oversized_json_store_only_safe_metadata() {
    for payload in [b"not-json".to_vec(), vec![b'x'; 1_048_577]] {
        let harness = Harness::new();
        let output = harness.run("codex", &payload);

        assert_eq!(output.status.code(), Some(2));
        assert!(!output.stderr.is_empty());
        let records = read_jsonl(harness.measure_root().join("invalid.jsonl"));
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["agent"], "codex");
        assert_eq!(records[0]["size"], payload.len());
        assert_eq!(records[0]["sha256"].as_str().unwrap().len(), 64);
        assert!(records[0].get("payload").is_none());
        assert!(harness.runs().is_empty());
    }
}

#[test]
fn falls_back_to_home_local_state_when_xdg_state_is_absent() {
    let harness = Harness::new();
    let mut command = harness.command("codex");
    command.env_remove("XDG_STATE_HOME");
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session","event":"SessionStart"}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_success(&output);
    assert!(
        harness
            .home
            .join(".local/state/dotfiles/agent-harness/runs")
            .is_dir()
    );
}

#[test]
fn refuses_relative_state_and_state_inside_the_observed_repository() {
    let harness = Harness::new();
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    let mut relative = harness.command("codex");
    relative.env("XDG_STATE_HOME", "relative/state");
    let mut child = relative.spawn().unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("absolute")
    );

    let mut inside = harness.command("codex");
    inside.env("XDG_STATE_HOME", harness.repository.join("state"));
    let mut child = inside.spawn().unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("repository")
    );
}

#[test]
fn refuses_state_inside_git_root_when_git_is_unavailable_from_a_subdirectory() {
    let harness = Harness::new();
    git(&harness.repository, &["init"]);
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["measure", "hook", "--agent", "codex"])
        .current_dir(nested)
        .env_clear()
        .env("HOME", &harness.home)
        .env("PATH", "/nonexistent")
        .env("XDG_STATE_HOME", harness.repository.join("state"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session"}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(!harness.repository.join("state").exists());
}

#[test]
fn nested_fake_git_marker_cannot_shrink_the_protected_repository() {
    let harness = Harness::new();
    git(&harness.repository, &["init"]);
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::create_dir(nested.join(".git")).unwrap();
    let current = nested.join("deeper");
    fs::create_dir(&current).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["measure", "hook", "--agent", "codex"])
        .current_dir(current)
        .env_clear()
        .env("HOME", &harness.home)
        .env("PATH", "/nonexistent")
        .env("XDG_STATE_HOME", harness.repository.join("state"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session"}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_eq!(output.status.code(), Some(2));
    assert!(!harness.repository.join("state").exists());
}

#[test]
fn creates_managed_directories_and_files_with_private_modes() {
    let harness = Harness::new();
    assert_success(&harness.run(
        "codex",
        br#"{"session_id":"session","event":"SessionStart","prompt":"hello"}"#,
    ));

    assert_private_tree(&harness.measure_root());
}

fn assert_private_tree(root: &Path) {
    for entry in walk(root) {
        let metadata = fs::symlink_metadata(&entry).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        if metadata.is_dir() {
            assert_eq!(mode, 0o700, "directory mode for {entry:?}");
        } else {
            assert_eq!(mode, 0o600, "file mode for {entry:?}");
        }
    }
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut paths = vec![root.to_owned()];
    let mut index = 0;
    while let Some(path) = paths.get(index).cloned() {
        index += 1;
        if path.is_dir() {
            paths.extend(
                fs::read_dir(path)
                    .unwrap()
                    .map(|entry| entry.unwrap().path()),
            );
        }
    }
    paths
}

#[test]
fn refuses_directory_and_file_symlink_attacks() {
    let harness = Harness::new();
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), harness.state.join("dotfiles")).unwrap();
    let output = harness.run(
        "codex",
        br#"{"session_id":"session","event":"SessionStart"}"#,
    );
    assert_eq!(output.status.code(), Some(2));
    assert!(fs::read_dir(outside.path()).unwrap().next().is_none());

    fs::remove_file(harness.state.join("dotfiles")).unwrap();
    assert_success(&harness.run(
        "codex",
        br#"{"session_id":"session","event":"SessionStart"}"#,
    ));
    let events = harness.only_run().join("events.jsonl");
    fs::remove_file(&events).unwrap();
    let captured = outside.path().join("captured");
    fs::write(&captured, "sentinel").unwrap();
    symlink(&captured, &events).unwrap();
    let output = harness.run("codex", br#"{"session_id":"session","event":"Followup"}"#);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(captured).unwrap(), "sentinel");
}

#[test]
fn refuses_hardlinked_and_corrupt_managed_files() {
    let harness = Harness::new();
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    assert_success(&harness.run("codex", payload));
    let run = harness.only_run();
    let run_json = run.join("run.json");
    let original_run = fs::read(&run_json).unwrap();
    fs::write(&run_json, "null").unwrap();
    let output = harness.run("codex", payload);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&run_json).unwrap(), "null");
    fs::write(&run_json, original_run).unwrap();

    let events = run.join("events.jsonl");
    fs::remove_file(&events).unwrap();
    let outside = harness.repository.join("tracked");
    fs::write(&outside, "sentinel").unwrap();
    fs::hard_link(&outside, &events).unwrap();
    let output = harness.run("codex", payload);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(&outside).unwrap(), "sentinel");

    fs::remove_file(&events).unwrap();
    fs::write(&events, r#"{"partial"#).unwrap();
    let output = harness.run("codex", payload);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fs::read_to_string(events).unwrap(), r#"{"partial"#);
}

#[test]
fn refuses_a_broken_run_json_symlink_instead_of_replacing_it() {
    let harness = Harness::new();
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    assert_success(&harness.run("codex", payload));
    let run_json = harness.only_run().join("run.json");
    fs::remove_file(&run_json).unwrap();
    symlink("missing-target", &run_json).unwrap();

    let output = harness.run("codex", payload);

    assert_eq!(output.status.code(), Some(2));
    assert_eq!(
        fs::read_link(run_json).unwrap(),
        Path::new("missing-target")
    );
}

#[test]
fn recursively_redacts_secret_keys_tokens_and_internal_reasoning() {
    let harness = Harness::new();
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"UserPromptSubmit sk-eventnamesecret",
        "model":"sk-modelnamesecret",
        "prompt":"use Bearer abc.def.ghi, sk-abcdefghijklmnopqrstuvwxyz, AKIAABCDEFGHIJKLMNOP, and glpat-abcdefghijklmnop",
        "api_key":"plain-secret",
        "oauth":{"client_secret":"nested-client-secret"},
        "aws_secret_access_key":"aws-secret-value",
        "reasoning_content":"reasoning detail",
        "analysis":"analysis detail",
        "request_id":"sk-requestidentifiersecret",
        "nested":{"password":"hunter2","reasoning":"private chain"},
        "safe":"visible"
    });
    assert_success(&harness.run("codex", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let events = read_jsonl(run.join("events.jsonl"));
    let artifact = fs::read_to_string(run.join(events[0]["artifact"].as_str().unwrap())).unwrap();
    let normalized = fs::read_to_string(run.join("events.jsonl")).unwrap();
    let prompts = fs::read_to_string(run.join("prompts.jsonl")).unwrap();
    let run_record = fs::read_to_string(run.join("run.json")).unwrap();
    for secret in [
        "abc.def.ghi",
        "sk-abcdefghijklmnopqrstuvwxyz",
        "AKIAABCDEFGHIJKLMNOP",
        "glpat-abcdefghijklmnop",
        "plain-secret",
        "nested-client-secret",
        "aws-secret-value",
        "reasoning detail",
        "analysis detail",
        "sk-requestidentifiersecret",
        "sk-modelnamesecret",
        "sk-eventnamesecret",
        "hunter2",
        "private chain",
    ] {
        assert!(!artifact.contains(secret), "artifact leaked {secret}");
        assert!(!normalized.contains(secret), "event log leaked {secret}");
        assert!(!prompts.contains(secret), "prompt log leaked {secret}");
        assert!(!run_record.contains(secret), "run record leaked {secret}");
    }
    assert!(artifact.contains("visible"));
    assert!(artifact.contains("[REDACTED]"));
}

#[test]
fn parallel_hooks_append_complete_json_lines() {
    let harness = Harness::new();
    let children: Vec<Child> = (0..24)
        .map(|index| {
            let mut child = harness.command("cursor").spawn().unwrap();
            let payload = json!({
                "conversation_id":"shared-session",
                "hook_event_name":"beforeSubmitPrompt",
                "prompt":format!("prompt-{index}"),
                "generation_id":format!("generation-{index}")
            });
            child
                .stdin
                .take()
                .unwrap()
                .write_all(payload.to_string().as_bytes())
                .unwrap();
            child
        })
        .collect();
    for child in children {
        assert_success(&child.wait_with_output().unwrap());
    }

    let run = harness.only_run();
    let events = read_jsonl(run.join("events.jsonl"));
    let prompts = read_jsonl(run.join("prompts.jsonl"));
    assert_eq!(events.len(), 24);
    assert_eq!(prompts.len(), 24);
    let mut ids: Vec<&str> = events
        .iter()
        .map(|event| event["event_id"].as_str().unwrap())
        .collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 24);
}

#[test]
fn records_git_metadata_model_and_deployed_harness_fingerprint() {
    let harness = Harness::new();
    git(&harness.repository, &["init", "-b", "measurement"]);
    fs::write(harness.repository.join("tracked"), "tracked").unwrap();
    git(&harness.repository, &["add", "tracked"]);
    git(
        &harness.repository,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            "initial",
        ],
    );
    fs::write(harness.repository.join("dirty"), "dirty").unwrap();
    fs::create_dir(harness.home.join(".codex")).unwrap();
    fs::write(harness.home.join(".codex/config.toml"), "model='one'").unwrap();
    let first = json!({"session_id":"one","event":"SessionStart","model":"gpt-test"});
    assert_success(&harness.run("codex", first.to_string().as_bytes()));
    let first_run = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .next()
        .unwrap();
    fs::write(harness.home.join(".codex/config.toml"), "model='two'").unwrap();
    let second = json!({"session_id":"two","event":"SessionStart"});
    assert_success(&harness.run("codex", second.to_string().as_bytes()));
    let runs: Vec<Value> = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .collect();
    let second_run = runs.iter().find(|run| run["session_id"] == "two").unwrap();

    assert_eq!(first_run["model"], "gpt-test");
    assert_eq!(
        first_run["repository"]["root"],
        fs::canonicalize(&harness.repository)
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(first_run["repository"]["branch"], "measurement");
    assert_eq!(first_run["repository"]["dirty"], true);
    assert_eq!(first_run["repository"]["head"].as_str().unwrap().len(), 40);
    assert_ne!(
        first_run["harness_fingerprint"],
        second_run["harness_fingerprint"]
    );
    assert!(!first_run.to_string().contains("model='one'"));
}

#[test]
fn fingerprint_tracks_project_instructions_config_hooks_and_skills() {
    let harness = Harness::new();
    fs::write(harness.repository.join("AGENTS.md"), "first instructions").unwrap();
    fs::create_dir(harness.repository.join(".codex")).unwrap();
    fs::write(
        harness.repository.join(".codex/config.toml"),
        "project='first'",
    )
    .unwrap();
    fs::write(
        harness.repository.join(".codex/hooks.json"),
        r#"{"hooks":{}}"#,
    )
    .unwrap();
    fs::create_dir_all(harness.repository.join(".codex/skills/example")).unwrap();
    fs::write(
        harness.repository.join(".codex/skills/example/SKILL.md"),
        "first skill",
    )
    .unwrap();
    assert_success(&harness.run("codex", br#"{"session_id":"one","event":"SessionStart"}"#));
    let first = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "one")
        .unwrap();

    fs::write(harness.repository.join("AGENTS.md"), "second instructions").unwrap();
    assert_success(&harness.run("codex", br#"{"session_id":"two","event":"SessionStart"}"#));
    let second = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "two")
        .unwrap();

    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
    assert!(!first.to_string().contains("first instructions"));
}

#[test]
fn fingerprint_tracks_each_agents_static_plugin_surface() {
    for (agent, session_key) in [
        ("codex", "session_id"),
        ("claude-code", "session_id"),
        ("cursor", "conversation_id"),
    ] {
        let harness = Harness::new();
        let plugin = configure_active_plugin(&harness, agent);
        fs::create_dir_all(plugin.parent().unwrap()).unwrap();
        fs::write(&plugin, "first plugin deployment").unwrap();
        let mut first_payload = json!({});
        first_payload[session_key] = json!("one");
        assert_success(&harness.run(agent, first_payload.to_string().as_bytes()));
        let first = harness
            .runs()
            .into_iter()
            .map(|path| read_json(path.join("run.json")))
            .find(|run| run["session_id"] == "one")
            .unwrap();
        if agent == "codex" {
            assert!(
                first["harness_fingerprint_limitations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| value.as_str().unwrap().contains("cache candidates"))
            );
        }
        fs::write(&plugin, "second plugin deployment").unwrap();
        let mut second_payload = json!({});
        second_payload[session_key] = json!("two");
        assert_success(&harness.run(agent, second_payload.to_string().as_bytes()));
        let second = harness
            .runs()
            .into_iter()
            .map(|path| read_json(path.join("run.json")))
            .find(|run| run["session_id"] == "two")
            .unwrap();

        assert_ne!(
            first["harness_fingerprint"], second["harness_fingerprint"],
            "{agent} plugin deployment was omitted"
        );
    }
}

fn configure_active_plugin(harness: &Harness, agent: &str) -> PathBuf {
    match agent {
        "codex" => {
            fs::create_dir_all(harness.home.join(".codex")).unwrap();
            fs::write(
                harness.home.join(".codex/config.toml"),
                "[plugins.\"demo@marketplace\"]\nenabled = true\n",
            )
            .unwrap();
            harness
                .home
                .join(".codex/plugins/cache/marketplace/demo/1.0.0/plugin.json")
        }
        "claude-code" => {
            let root = harness
                .home
                .join(".claude/plugins/cache/marketplace/demo/1.0.0");
            fs::create_dir_all(harness.home.join(".claude/plugins")).unwrap();
            fs::write(
                harness.home.join(".claude/plugins/installed_plugins.json"),
                json!({
                    "version":2,
                    "plugins":{"demo@marketplace":[{
                        "scope":"user",
                        "installPath":root,
                        "version":"1.0.0"
                    }]}
                })
                .to_string(),
            )
            .unwrap();
            root.join("plugin.json")
        }
        "cursor" => harness.home.join(".cursor/plugins/local/demo/plugin.json"),
        _ => unreachable!(),
    }
}

#[test]
fn fingerprint_includes_the_first_512_sorted_deployment_entries() {
    let harness = Harness::new();
    let skills = harness.home.join(".agents/skills");
    fs::create_dir_all(&skills).unwrap();
    for index in 0..400 {
        fs::write(
            skills.join(format!("skill-{index:03}")),
            format!("value-{index}"),
        )
        .unwrap();
    }
    assert_success(&harness.run("codex", br#"{"session_id":"one"}"#));
    let first = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "one")
        .unwrap();
    fs::write(skills.join("skill-300"), "changed").unwrap();
    assert_success(&harness.run("codex", br#"{"session_id":"two"}"#));
    let second = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "two")
        .unwrap();

    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn fingerprint_refuses_deployments_exceeding_512_entries() {
    let harness = Harness::new();
    let skills = harness.home.join(".agents/skills");
    fs::create_dir_all(&skills).unwrap();
    for index in 0..513 {
        fs::write(skills.join(format!("skill-{index:03}")), "value").unwrap();
    }

    let output = harness.run("codex", br#"{"session_id":"session"}"#);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("fingerprint inventory exceeds 512 entries")
    );
}

#[test]
fn fingerprint_refuses_more_than_512_registered_plugin_file_roots() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins).unwrap();
    let mut registered = serde_json::Map::new();
    for index in 0..513 {
        let plugin = plugins.join(format!("plugin-{index:03}.json"));
        fs::write(&plugin, "plugin").unwrap();
        registered.insert(
            format!("plugin-{index:03}@marketplace"),
            json!([{"installPath":plugin,"version":"1.0.0"}]),
        );
    }
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({"version":2,"plugins":registered}).to_string(),
    )
    .unwrap();

    let output = harness.run("claude-code", br#"{"session_id":"session"}"#);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("fingerprint inventory exceeds 512 entries")
    );
}

#[test]
fn fingerprint_counts_registered_plugin_aliases_against_the_global_limit() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins).unwrap();
    let plugin = plugins.join("shared.json");
    fs::write(&plugin, "plugin").unwrap();
    let mut registered = serde_json::Map::new();
    for index in 0..513 {
        registered.insert(
            format!("plugin-{index:03}@marketplace"),
            json!([{"installPath":plugin,"version":"1.0.0"}]),
        );
    }
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({"version":2,"plugins":registered}).to_string(),
    )
    .unwrap();

    let output = harness.run("claude-code", br#"{"session_id":"session"}"#);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("fingerprint inventory exceeds 512 entries")
    );
}

#[test]
fn fingerprint_refuses_oversized_plugin_manifests() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins).unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        vec![b' '; 1_048_577],
    )
    .unwrap();

    let output = harness.run("claude-code", br#"{"session_id":"session"}"#);

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("plugin manifest exceeds 1048576 bytes")
    );
}

fn git(repository: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
