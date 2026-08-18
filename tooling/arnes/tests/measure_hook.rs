use serde_json::{Value, json};
use std::fs;
use std::io::{Seek, SeekFrom, Write};
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

    fn list(&self) -> Output {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["measure", "list", "--format", "json"])
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .output()
            .unwrap()
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
fn compact_large_artifact_remains_capturable_and_listable() {
    let harness = Harness::new();
    let payload = json!({
        "session_id": "compact-large",
        "hook_event_name": "FutureEvent",
        "data": vec![0; 250_000]
    });
    let compact = serde_json::to_vec(&payload).unwrap();
    assert!(compact.len() < 1_048_576);
    assert!(serde_json::to_vec_pretty(&payload).unwrap().len() > 1_100_000);

    assert_success(&harness.run("codex", &compact));

    let run = harness.only_run();
    let artifact = fs::read_dir(run.join("artifacts/hooks"))
        .unwrap()
        .next()
        .unwrap()
        .unwrap()
        .path();
    let artifact = fs::read(artifact).unwrap();
    assert!(artifact.len() <= 1_100_000);
    assert_eq!(serde_json::from_slice::<Value>(&artifact).unwrap(), payload);
    let listed = harness.list();
    assert_eq!(listed.status.code(), Some(0));
    assert!(listed.stderr.is_empty());
    let listed: Value = serde_json::from_slice(&listed.stdout).unwrap();
    assert_eq!(listed.as_array().unwrap().len(), 1);
}

#[test]
fn expanded_artifact_is_rejected_without_a_partial_run() {
    let harness = Harness::new();
    let payload = json!({
        "session_id": "expanded-large",
        "hook_event_name": "FutureEvent",
        "data": vec![json!({"token": "x"}); 65_000]
    });
    let compact = serde_json::to_vec(&payload).unwrap();
    assert!(compact.len() < 1_048_576);

    let output = harness.run("codex", &compact);

    assert_advisory_failure(&output);
    assert!(harness.runs().is_empty());
    let invalid = read_jsonl(harness.measure_root().join("invalid.jsonl"));
    assert_eq!(invalid.len(), 1);
    assert!(
        invalid[0]["error"]
            .as_str()
            .unwrap()
            .contains("exceeds 1100000 bytes")
    );
}

#[path = "measure_hook/event_names.rs"]
mod event_names;

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
fn home_without_git_can_use_default_state_below_home() {
    let harness = Harness::new();
    let mut command = harness.command("codex");
    command
        .current_dir(&harness.home)
        .env_remove("XDG_STATE_HOME");
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session"}"#)
        .unwrap();

    assert_success(&child.wait_with_output().unwrap());
    assert!(
        harness
            .home
            .join(".local/state/dotfiles/agent-harness/runs")
            .is_dir()
    );
}

#[test]
fn recursively_duplicate_json_keys_are_advisory_and_never_create_a_run() {
    let harness = Harness::new();
    let payload = br#"{"session_id":"session","nested":{"value":1,"value":2}}"#;

    let output = harness.run("codex", payload);

    assert_advisory_failure(&output);
    assert!(harness.runs().is_empty());
    let invalid = read_jsonl(harness.measure_root().join("invalid.jsonl"));
    assert_eq!(invalid.len(), 1);
    assert_eq!(invalid[0]["size"], payload.len());
    assert!(invalid[0]["error"].as_str().unwrap().contains("duplicate"));
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

    git(&harness.repository, &["init"]);
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
    init_repository(&harness.repository, "outer", "outer-file");
    fs::write(harness.repository.join("AGENTS.md"), "outer one").unwrap();
    let inner = harness.repository.join("inner");
    fs::create_dir(&inner).unwrap();
    init_repository(&inner, "inner", "inner-file");
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

#[path = "measure_hook/collection_boundaries.rs"]
mod collection_boundaries;

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
    init_repository(&harness.repository, "measurement", "tracked");
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

#[path = "measure_hook/fingerprint.rs"]
mod fingerprint;

fn init_repository(repository: &Path, branch: &str, tracked: &str) {
    git(repository, &["init", "-b", branch]);
    fs::write(repository.join(tracked), tracked).unwrap();
    git(repository, &["add", tracked]);
    commit(repository, "initial");
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
