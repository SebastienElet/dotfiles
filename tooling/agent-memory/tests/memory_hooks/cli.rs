#[path = "support.rs"]
mod support;

use serde_json::Value;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::{Command, Stdio};
use support::*;

#[test]
fn codex_and_claude_hooks_retrieve_through_the_real_binary() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft(
        "invariant",
        "Durable hook architecture.",
        "durable hook architecture",
    );
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft), 0);

    for agent in ["codex", "claude"] {
        let payload = payload(
            "UserPromptSubmit",
            "Apply durable hook architecture",
            fixture.repository(),
        );
        let output = fixture.run(["hook", "--agent", agent], &payload);

        assert_exit(&output, 0);
        let response = stdout_json(&output);
        assert_eq!(
            response["hookSpecificOutput"]["hookEventName"],
            "UserPromptSubmit"
        );
        let context = response["hookSpecificOutput"]["additionalContext"]
            .as_str()
            .unwrap();
        assert!(context.starts_with("AGENT_MEMORY_CONTEXT_V1\n"));
        assert!(context.contains("Durable hook architecture."));
        assert!(!context.contains("mem_"));
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn an_unrelated_prompt_returns_no_additional_context() {
    let fixture = CliFixture::new();
    let draft = fixture.git_draft(
        "invariant",
        "Durable hook architecture.",
        "durable hook architecture",
    );
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft), 0);
    let payload = payload(
        "UserPromptSubmit",
        "unrelated request",
        fixture.repository(),
    );

    let output = fixture.run(["hook", "--agent", "codex"], &payload);

    assert_exit(&output, 0);
    assert_eq!(stdout_json(&output), Value::Object(Default::default()));
}

#[test]
fn invalid_payloads_use_exit_two_without_echoing_input() {
    let fixture = CliFixture::new();
    let private = "ghp_abcdefghijklmnopqrstuvwxyz1234567890";
    let cases = [
        Vec::new(),
        format!("{{\"prompt\":\"{private}\"").into_bytes(),
        payload("Stop", private, fixture.repository()),
        serde_json::to_vec(&serde_json::json!({
            "hook_event_name": "UserPromptSubmit",
            "cwd": fixture.repository(),
        }))
        .unwrap(),
    ];

    for bytes in cases {
        let output = fixture.run(["hook", "--agent", "claude"], &bytes);
        assert_exit(&output, 2);
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains(private));
    }

    let oversized = vec![b'a'; 1024 * 1024 + 1];
    assert_error(
        &fixture.run(["hook", "--agent", "codex"], &oversized),
        2,
        "input_too_large",
    );
}

#[test]
fn an_unavailable_project_git_process_exits_four_without_context() {
    let fixture = CliFixture::new();
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable hook architecture",
        fixture.repository(),
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
        .args(["hook", "--agent", "codex"])
        .current_dir(fixture.repository())
        .env("AGENT_MEMORY_ROOT", fixture.root())
        .env("PATH", fixture.root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), &payload).unwrap();
    let output = child.wait_with_output().unwrap();

    assert_error(&output, 4, "scope_unavailable");
}

#[test]
fn cursor_is_not_a_native_hook_variant() {
    let fixture = CliFixture::new();
    let output = fixture.run(["hook", "--agent", "cursor"], b"{}");

    assert_error(&output, 2, "invalid_arguments");
}

#[test]
fn an_unavailable_store_never_reuses_prior_context() {
    let fixture = CliFixture::new();
    let statement = "Durable unavailable store memory.";
    let draft = fixture.git_draft("invariant", statement, "durable unavailable store");
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft), 0);
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable unavailable store",
        fixture.repository(),
    );
    assert_exit(&fixture.run(["hook", "--agent", "codex"], &payload), 0);
    fs::set_permissions(fixture.root(), fs::Permissions::from_mode(0o755)).unwrap();

    let output = fixture.run(["hook", "--agent", "codex"], &payload);

    assert_exit(&output, 4);
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains(statement));
}

#[test]
fn an_unavailable_selected_yaml_exits_four_without_context() {
    let fixture = CliFixture::new();
    let statement = "Durable selected path memory.";
    let draft = fixture.git_draft("invariant", statement, "durable selected path");
    let admitted = fixture.run(["admit", "--format", "json"], &draft);
    assert_exit(&admitted, 0);
    let id = stdout_json(&admitted)["id"].as_str().unwrap().to_owned();
    let path = find_entry(fixture.root(), &format!("{id}.yaml")).unwrap();
    let displaced = fixture.root().join("displaced-selected.yaml");
    fs::rename(&path, &displaced).unwrap();
    symlink(&displaced, &path).unwrap();
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable selected path",
        fixture.repository(),
    );

    let output = fixture.run(["hook", "--agent", "claude"], &payload);

    assert_error(&output, 4, "unsafe_store_path");
    assert!(!String::from_utf8_lossy(&output.stderr).contains(statement));
}

#[test]
fn a_hook_stdout_failure_uses_exit_four_without_context_on_stderr() {
    let fixture = CliFixture::new();
    let statement = "Durable stdout memory.";
    let draft = fixture.git_draft("invariant", statement, "durable stdout memory");
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft), 0);
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable stdout memory",
        fixture.repository(),
    );
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
        .args(["hook", "--agent", "claude"])
        .current_dir(fixture.repository())
        .env("AGENT_MEMORY_ROOT", fixture.root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    drop(child.stdout.take());
    std::io::Write::write_all(child.stdin.as_mut().unwrap(), &payload).unwrap();
    let output = child.wait_with_output().unwrap();

    assert_exit(&output, 4);
    let diagnostic: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(diagnostic["error"]["code"], "output_unavailable");
    assert!(!String::from_utf8_lossy(&output.stderr).contains(statement));
}

fn payload(event: &str, prompt: &str, cwd: &std::path::Path) -> Vec<u8> {
    serde_json::to_vec(&serde_json::json!({
        "session_id": "session-fixture",
        "transcript_path": "/private/transcript.jsonl",
        "cwd": cwd,
        "permission_mode": "default",
        "hook_event_name": event,
        "prompt": prompt,
    }))
    .unwrap()
}

fn find_entry(directory: &std::path::Path, name: &str) -> Option<std::path::PathBuf> {
    for entry in fs::read_dir(directory).ok()? {
        let path = entry.ok()?.path();
        if path.is_dir() {
            if let Some(found) = find_entry(&path, name) {
                return Some(found);
            }
        } else if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            return Some(path);
        }
    }
    None
}
