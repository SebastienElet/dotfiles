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
        Self::with_repository_name("repository")
    }

    fn with_repository_name(name: &str) -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repository = root.path().join(name);
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

fn assert_advisory_failure(output: &Output) {
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}

fn run_at(harness: &Harness, current: &Path, state: &Path, payload: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["measure", "hook", "--agent", "codex"])
        .current_dir(current)
        .env_clear()
        .env("HOME", &harness.home)
        .env("XDG_STATE_HOME", state)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();
    child.wait_with_output().unwrap()
}

fn run_record(harness: &Harness, session: &str) -> Value {
    harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == session)
        .unwrap()
}

fn capture_run(harness: &Harness, agent: &str, session_key: &str, session: &str) -> Value {
    let mut payload = json!({});
    payload[session_key] = json!(session);
    assert_success(&harness.run(agent, payload.to_string().as_bytes()));
    run_record(harness, session)
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

    assert_advisory_failure(&output);
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

        assert_advisory_failure(&output);
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
    assert_advisory_failure(&output);
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
    assert_advisory_failure(&output);
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

    assert_advisory_failure(&output);
    assert!(!harness.repository.join("state").exists());
}

#[test]
fn refuses_state_inside_repository_observed_only_through_git_environment() {
    let harness = Harness::new();
    let git_dir = harness._root.path().join("external.git");
    git(
        &harness.repository,
        &["init", "--bare", git_dir.to_str().unwrap()],
    );
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested).unwrap();
    let state = harness.repository.join("state");
    let mut command = harness.command("codex");
    command
        .current_dir(nested)
        .env("GIT_DIR", &git_dir)
        .env("GIT_WORK_TREE", &harness.repository)
        .env("XDG_STATE_HOME", &state);
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session"}"#)
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert_advisory_failure(&output);
    assert!(!state.exists());
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

    assert_advisory_failure(&output);
    assert!(!harness.repository.join("state").exists());
}

#[test]
fn nested_git_repository_is_observed_while_both_repository_boundaries_are_protected() {
    let harness = Harness::new();
    git(&harness.repository, &["init", "-b", "outer"]);
    fs::write(harness.repository.join("outer-file"), "outer").unwrap();
    git(&harness.repository, &["add", "outer-file"]);
    commit(&harness.repository, "outer");
    fs::write(harness.repository.join("AGENTS.md"), "outer one").unwrap();
    let inner = harness.repository.join("inner");
    fs::create_dir(&inner).unwrap();
    git(&inner, &["init", "-b", "inner"]);
    fs::write(inner.join("inner-file"), "inner").unwrap();
    git(&inner, &["add", "inner-file"]);
    commit(&inner, "inner");
    fs::write(inner.join("AGENTS.md"), "inner one").unwrap();

    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"one"}"#,
    ));
    let first = run_record(&harness, "one");
    assert_eq!(
        first["repository"],
        fs::canonicalize(&inner).unwrap().to_str().unwrap()
    );
    assert_eq!(
        first["repository_commit"],
        git_value(&inner, &["rev-parse", "HEAD"])
    );
    assert_eq!(first["repository_branch"], "inner");

    fs::write(harness.repository.join("AGENTS.md"), "outer two").unwrap();
    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"two"}"#,
    ));
    let second = run_record(&harness, "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(inner.join("AGENTS.md"), "inner two").unwrap();
    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"three"}"#,
    ));
    let third = run_record(&harness, "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);

    for state in [inner.join("state"), harness.repository.join("state")] {
        let output = run_at(&harness, &inner, &state, br#"{"session_id":"blocked"}"#);
        assert_advisory_failure(&output);
        assert!(!state.exists());
    }
}

#[test]
fn git_repository_root_preserves_trailing_spaces() {
    let harness = Harness::with_repository_name("repository ");
    git(&harness.repository, &["init", "-b", "measurement"]);
    fs::write(harness.repository.join("tracked"), "tracked").unwrap();
    git(&harness.repository, &["add", "tracked"]);
    commit(&harness.repository, "initial");
    fs::write(harness.repository.join("AGENTS.md"), "one").unwrap();

    let first = capture_run(&harness, "codex", "session_id", "one");
    assert_eq!(
        first["repository"],
        fs::canonicalize(&harness.repository)
            .unwrap()
            .to_str()
            .unwrap()
    );
    fs::write(harness.repository.join("AGENTS.md"), "two").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");
    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
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
    assert_advisory_failure(&output);
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
    assert_advisory_failure(&output);
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
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(&run_json).unwrap(), "null");
    fs::write(&run_json, original_run).unwrap();

    let events = run.join("events.jsonl");
    fs::remove_file(&events).unwrap();
    let outside = harness.repository.join("tracked");
    fs::write(&outside, "sentinel").unwrap();
    fs::hard_link(&outside, &events).unwrap();
    let output = harness.run("codex", payload);
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(&outside).unwrap(), "sentinel");

    fs::remove_file(&events).unwrap();
    fs::write(&events, r#"{"partial"#).unwrap();
    let output = harness.run("codex", payload);
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(events).unwrap(), r#"{"partial"#);
}

#[test]
fn refuses_incompletely_typed_run_records_without_appending() {
    for (field, replacement) in [
        ("model", Some(json!([]))),
        ("repository", Some(json!(["wrong"]))),
        ("model", None),
        ("harness_fingerprint_limitations", Some(json!(false))),
    ] {
        let harness = Harness::new();
        let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
        assert_success(&harness.run("codex", payload));
        let run = harness.only_run();
        let run_json = run.join("run.json");
        let events = run.join("events.jsonl");
        let artifacts = run.join("artifacts/hooks");
        let before_events = fs::read(&events).unwrap();
        let before_artifacts = fs::read_dir(&artifacts).unwrap().count();
        let mut record = read_json(&run_json);
        match replacement {
            Some(value) => record[field] = value,
            None => {
                record.as_object_mut().unwrap().remove(field);
            }
        }
        let corrupted = serde_json::to_vec(&record).unwrap();
        fs::write(&run_json, &corrupted).unwrap();
        assert_eq!(harness.runs().len(), 1);
        assert_eq!(fs::read(&run_json).unwrap(), corrupted);

        let output = harness.run("codex", payload);

        assert_eq!(harness.runs(), vec![run.clone()]);
        assert_eq!(fs::read(&run_json).unwrap(), corrupted, "field {field}");
        assert_advisory_failure(&output);
        assert_eq!(fs::read(&events).unwrap(), before_events, "field {field}");
        assert_eq!(
            fs::read_dir(artifacts).unwrap().count(),
            before_artifacts,
            "field {field}"
        );
    }
}

#[test]
fn refuses_duplicate_run_record_keys_without_appending() {
    let harness = Harness::new();
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    assert_success(&harness.run("codex", payload));
    let run = harness.only_run();
    let run_json = run.join("run.json");
    let events = run.join("events.jsonl");
    let artifacts = run.join("artifacts/hooks");
    let original = fs::read_to_string(&run_json).unwrap();
    let corrupted = original.replacen('{', r#"{"agent":"evil","#, 1);
    fs::write(&run_json, &corrupted).unwrap();
    let before_events = fs::read(&events).unwrap();
    let before_artifacts = fs::read_dir(&artifacts).unwrap().count();

    let output = harness.run("codex", payload);

    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(run_json).unwrap(), corrupted);
    assert_eq!(fs::read(events).unwrap(), before_events);
    assert_eq!(fs::read_dir(artifacts).unwrap().count(), before_artifacts);
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

    assert_advisory_failure(&output);
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
fn after_agent_thought_never_persists_reasoning_values() {
    let harness = Harness::new();
    let secret = "manually-injected-private-thought";
    let payload = json!({
        "session_id":"session",
        "hook_event_name":"afterAgentThought",
        "text":secret,
        "prompt":secret,
        "unclassified":secret,
        "thought":secret,
        "nested":{
            "thoughts":secret,
            "reasoning":secret,
            "chain_of_thought":secret
        }
    });

    assert_success(&harness.run("claude-code", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let artifact = fs::read_to_string(
        fs::read_dir(run.join("artifacts/hooks"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path(),
    )
    .unwrap();
    assert!(!artifact.contains(secret));
    assert!(!run.join("prompts.jsonl").exists());
}

#[test]
fn after_agent_response_preserves_non_reasoning_text() {
    let harness = Harness::new();
    let text = "visible-agent-response";
    let payload = json!({
        "conversation_id":"session",
        "hook_event_name":"afterAgentResponse",
        "text":text
    });

    assert_success(&harness.run("cursor", payload.to_string().as_bytes()));

    let run = harness.only_run();
    let artifact = fs::read_to_string(
        fs::read_dir(run.join("artifacts/hooks"))
            .unwrap()
            .next()
            .unwrap()
            .unwrap()
            .path(),
    )
    .unwrap();
    assert!(artifact.contains(text));
}

#[test]
fn collection_failure_after_an_existing_run_is_advisory_and_journaled() {
    let harness = Harness::new();
    assert_success(&harness.run("claude-code", br#"{"session_id":"existing"}"#));
    let skills = harness.home.join(".claude/skills");
    fs::create_dir_all(&skills).unwrap();
    for index in 0..513 {
        fs::write(skills.join(format!("skill-{index:03}")), "value").unwrap();
    }
    let payload = br#"{"session_id":"new","event":"SessionStart"}"#;

    let output = harness.run("claude-code", payload);

    assert_advisory_failure(&output);
    assert!(String::from_utf8(output.stderr).unwrap().contains("512"));
    let invalid = read_jsonl(harness.measure_root().join("invalid.jsonl"));
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["agent"], "claude-code");
    assert_eq!(invalid[0]["size"], payload.len());
    assert_eq!(invalid[0]["sha256"].as_str().unwrap().len(), 64);
    assert!(invalid[0].get("payload").is_none());
}

#[test]
fn subsequent_events_skip_immutable_collection_and_append_from_the_tail() {
    let harness = Harness::new();
    git(&harness.repository, &["init"]);
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    assert_success(&harness.run("codex", payload));
    let run = harness.only_run();
    let events = run.join("events.jsonl");
    let mut existing = fs::read(&events).unwrap();
    let mut prefixed = b"not-json\n".to_vec();
    prefixed.append(&mut existing);
    fs::write(&events, prefixed).unwrap();
    let skills = harness.home.join(".agents/skills");
    fs::create_dir_all(&skills).unwrap();
    for index in 0..513 {
        fs::write(skills.join(format!("skill-{index:03}")), "value").unwrap();
    }
    let mut command = harness.command("codex");
    command.env("PATH", "/nonexistent");
    let mut child = command.spawn().unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();

    let output = child.wait_with_output().unwrap();

    assert_success(&output);
    let lines = fs::read_to_string(events).unwrap();
    assert_eq!(lines.lines().count(), 3);
    serde_json::from_str::<Value>(lines.lines().last().unwrap()).unwrap();
    assert!(!harness.measure_root().join("invalid.jsonl").exists());
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
        first_run["repository"],
        fs::canonicalize(&harness.repository)
            .unwrap()
            .to_str()
            .unwrap()
    );
    assert_eq!(first_run["repository_branch"], "measurement");
    assert_eq!(first_run["repository_dirty"], true);
    assert_eq!(first_run["repository_commit"].as_str().unwrap().len(), 40);
    assert_eq!(first_run.as_object().unwrap().len(), 12);
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
fn claude_fingerprint_hashes_only_enabled_registered_plugins() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0/plugin.json");
    let disabled = plugins.join("cache/marketplace/disabled/1.0.0/plugin.json");
    fs::create_dir_all(active.parent().unwrap()).unwrap();
    fs::create_dir_all(disabled.parent().unwrap()).unwrap();
    fs::write(&active, "active one").unwrap();
    fs::write(&disabled, "disabled one").unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true,"disabled@marketplace":false}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":2,
            "plugins":{
                "active@marketplace":[{"scope":"user","installPath":active.parent().unwrap(),"version":"1.0.0"}],
                "disabled@marketplace":[{"scope":"user","installPath":disabled.parent().unwrap(),"version":"1.0.0"}]
            }
        })
        .to_string(),
    )
    .unwrap();

    let first = capture_run(&harness, "claude-code", "session_id", "one");
    fs::write(&disabled, "disabled two").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(&active, "active two").unwrap();
    let third = capture_run(&harness, "claude-code", "session_id", "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);
}

#[test]
fn claude_fingerprint_does_not_follow_active_plugin_symlinks_outside_the_installation() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active).unwrap();
    fs::write(active.join("plugin.json"), "plugin").unwrap();
    let transcript = harness.home.join("session.jsonl");
    fs::write(&transcript, "session one").unwrap();
    symlink(&transcript, active.join("session.jsonl")).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":2,
            "plugins":{"active@marketplace":[{"scope":"user","installPath":active,"version":"1.0.0"}]}
        })
        .to_string(),
    )
    .unwrap();

    let first = capture_run(&harness, "claude-code", "session_id", "one");
    fs::write(transcript, "session two").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");

    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn claude_fingerprint_refuses_oversized_active_plugin_files() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active).unwrap();
    fs::write(active.join("plugin.bin"), vec![b'x'; 1_048_577]).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":2,
            "plugins":{"active@marketplace":[{"scope":"user","installPath":active,"version":"1.0.0"}]}
        })
        .to_string(),
    )
    .unwrap();

    let output = harness.run("claude-code", br#"{"session_id":"session"}"#);

    assert_advisory_failure(&output);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("fingerprint file exceeds 1048576 bytes")
    );
}

#[test]
fn unsupported_claude_registry_excludes_plugin_files_with_a_limitation() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active).unwrap();
    fs::write(active.join("plugin.json"), "plugin one").unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":3,
            "plugins":{"active@marketplace":[{"scope":"user","installPath":active,"version":"1.0.0"}]}
        })
        .to_string(),
    )
    .unwrap();

    let first = capture_run(&harness, "claude-code", "session_id", "one");
    assert!(
        first["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("registry"))
    );
    fs::write(active.join("plugin.json"), "plugin two").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn codex_fingerprint_hashes_only_single_version_enabled_plugins() {
    let harness = Harness::new();
    fs::create_dir_all(harness.home.join(".codex")).unwrap();
    fs::write(
        harness.home.join(".codex/config.toml"),
        "[plugins.\"active@marketplace\"]\nenabled = true\n[plugins.\"disabled@marketplace\"]\nenabled = false\n",
    )
    .unwrap();
    let active = harness
        .home
        .join(".codex/plugins/cache/marketplace/active/1.0.0/plugin.json");
    let disabled = harness
        .home
        .join(".codex/plugins/cache/marketplace/disabled/1.0.0/plugin.json");
    fs::create_dir_all(active.parent().unwrap()).unwrap();
    fs::create_dir_all(disabled.parent().unwrap()).unwrap();
    fs::write(&active, "active one").unwrap();
    fs::write(&disabled, "disabled one").unwrap();

    let first = capture_run(&harness, "codex", "session_id", "one");
    fs::write(&disabled, "disabled two").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(&active, "active two").unwrap();
    let third = capture_run(&harness, "codex", "session_id", "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);
}

#[test]
fn codex_fingerprint_records_ambiguous_enabled_plugin_versions() {
    let harness = Harness::new();
    fs::create_dir_all(harness.home.join(".codex")).unwrap();
    fs::write(
        harness.home.join(".codex/config.toml"),
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    )
    .unwrap();
    let cache = harness.home.join(".codex/plugins/cache/marketplace/demo");
    for version in ["1.0.0", "2.0.0"] {
        fs::create_dir_all(cache.join(version)).unwrap();
        fs::write(cache.join(version).join("plugin.json"), version).unwrap();
    }

    let first = capture_run(&harness, "codex", "session_id", "one");
    assert!(
        first["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("ambiguous"))
    );
    fs::create_dir_all(cache.join("3.0.0")).unwrap();
    fs::write(cache.join("3.0.0/plugin.json"), "three").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");
    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn codex_ambiguous_version_sets_have_unambiguous_fingerprints() {
    let capture = |versions: &[&str]| {
        let harness = Harness::new();
        fs::create_dir_all(harness.home.join(".codex")).unwrap();
        fs::write(
            harness.home.join(".codex/config.toml"),
            "[plugins.\"demo@marketplace\"]\nenabled = true\n",
        )
        .unwrap();
        let cache = harness.home.join(".codex/plugins/cache/marketplace/demo");
        for version in versions {
            fs::create_dir_all(cache.join(version)).unwrap();
        }
        capture_run(&harness, "codex", "session_id", "session")["harness_fingerprint"]
            .as_str()
            .unwrap()
            .to_owned()
    };

    assert_ne!(capture(&["a,b", "c"]), capture(&["a", "b,c"]));
}

#[test]
fn codex_plugin_roots_cannot_escape_the_cache() {
    let escaped = Harness::new();
    fs::create_dir_all(escaped.home.join(".codex")).unwrap();
    fs::write(
        escaped.home.join(".codex/config.toml"),
        "[plugins.\"payload@../../../outside\"]\nenabled = true\n",
    )
    .unwrap();
    let escaped_file = escaped.home.join("outside/payload/1.0.0/plugin.json");
    fs::create_dir_all(escaped_file.parent().unwrap()).unwrap();
    fs::write(&escaped_file, "one").unwrap();
    let first = capture_run(&escaped, "codex", "session_id", "one");
    fs::write(&escaped_file, "two").unwrap();
    let second = capture_run(&escaped, "codex", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);

    let linked = Harness::new();
    fs::create_dir_all(linked.home.join(".codex/plugins/cache/marketplace")).unwrap();
    fs::write(
        linked.home.join(".codex/config.toml"),
        "[plugins.\"active@marketplace\"]\nenabled = true\n",
    )
    .unwrap();
    let outside = linked.home.join("outside/active");
    let linked_file = outside.join("1.0.0/plugin.json");
    fs::create_dir_all(linked_file.parent().unwrap()).unwrap();
    fs::write(&linked_file, "one").unwrap();
    symlink(
        &outside,
        linked.home.join(".codex/plugins/cache/marketplace/active"),
    )
    .unwrap();
    let first = capture_run(&linked, "codex", "session_id", "one");
    fs::write(&linked_file, "two").unwrap();
    let second = capture_run(&linked, "codex", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn cursor_fingerprint_hashes_only_declared_local_plugins() {
    let harness = Harness::new();
    fs::write(
        harness.home.join(".arnes.yaml"),
        "version: 1\nagents:\n  - id: cursor\n    scopes: [user]\nexternal:\n  roots: []\n  plugins:\n    - { agent: cursor, scope: user, id: active }\n  skills: []\nresources: []\n",
    )
    .unwrap();
    let active = harness
        .home
        .join(".cursor/plugins/local/active/plugin.json");
    let inactive = harness
        .home
        .join(".cursor/plugins/local/inactive/plugin.json");
    fs::create_dir_all(active.parent().unwrap()).unwrap();
    fs::create_dir_all(inactive.parent().unwrap()).unwrap();
    fs::write(&active, "active one").unwrap();
    fs::write(&inactive, "inactive one").unwrap();

    let first = capture_run(&harness, "cursor", "conversation_id", "one");
    fs::write(&inactive, "inactive two").unwrap();
    let second = capture_run(&harness, "cursor", "conversation_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(&active, "active two").unwrap();
    let third = capture_run(&harness, "cursor", "conversation_id", "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);
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

    assert_advisory_failure(&output);
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
    let mut enabled = serde_json::Map::new();
    for index in 0..513 {
        let plugin = plugins.join(format!("plugin-{index:03}.json"));
        fs::write(&plugin, "plugin").unwrap();
        let id = format!("plugin-{index:03}@marketplace");
        registered.insert(
            id.clone(),
            json!([{"installPath":plugin,"version":"1.0.0"}]),
        );
        enabled.insert(id, json!(true));
    }
    fs::create_dir_all(harness.home.join(".claude")).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        json!({"enabledPlugins":enabled}).to_string(),
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({"version":2,"plugins":registered}).to_string(),
    )
    .unwrap();

    let output = harness.run("claude-code", br#"{"session_id":"session"}"#);

    assert_advisory_failure(&output);
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
    let mut enabled = serde_json::Map::new();
    for index in 0..513 {
        let id = format!("plugin-{index:03}@marketplace");
        registered.insert(
            id.clone(),
            json!([{"installPath":plugin,"version":"1.0.0"}]),
        );
        enabled.insert(id, json!(true));
    }
    fs::create_dir_all(harness.home.join(".claude")).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        json!({"enabledPlugins":enabled}).to_string(),
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({"version":2,"plugins":registered}).to_string(),
    )
    .unwrap();

    let output = harness.run("claude-code", br#"{"session_id":"session"}"#);

    assert_advisory_failure(&output);
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

    assert_advisory_failure(&output);
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

fn commit(repository: &Path, message: &str) {
    git(
        repository,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            message,
        ],
    );
}

fn git_value(repository: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap().trim().to_owned()
}
