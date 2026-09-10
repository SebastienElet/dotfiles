#[path = "support.rs"]
mod support;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use std::process::{Command, Stdio};
use support::*;
#[test]
fn codex_and_claude_hooks_retrieve_through_the_real_binary()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft(
        "invariant",
        "Durable hook architecture.",
        "durable hook architecture",
    )?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft)?, 0);
    let _: () = for agent in ["codex", "claude"] {
        let payload = payload(
            "UserPromptSubmit",
            "Apply durable hook architecture",
            fixture.repository(),
        )?;
        let output = fixture.run(["hook", "--agent", agent], &payload)?;
        assert_exit(&output, 0);
        let response = stdout_json(&output)?;
        assert_eq!(
            (*response
                .pointer("/hookSpecificOutput/hookEventName")
                .ok_or("missing hook response /hookSpecificOutput/hookEventName")?),
            "UserPromptSubmit"
        );
        let context = (*response
            .pointer("/hookSpecificOutput/additionalContext")
            .ok_or("missing hook response /hookSpecificOutput/additionalContext")?)
        .as_str()
        .ok_or("expected JSON string")?;
        assert!(context.starts_with("AGENT_MEMORY_CONTEXT_V1\n"));
        assert!(context.contains("Durable hook architecture."));
        assert!(!context.contains("mem_"));
        assert!(output.stderr.is_empty());
    };
    Ok(())
}
#[test]
fn an_unrelated_prompt_returns_no_additional_context()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft(
        "invariant",
        "Durable hook architecture.",
        "durable hook architecture",
    )?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft)?, 0);
    let payload = payload(
        "UserPromptSubmit",
        "unrelated request",
        fixture.repository(),
    )?;
    let output = fixture.run(["hook", "--agent", "codex"], &payload)?;
    assert_exit(&output, 0);
    assert_eq!(
        stdout_json(&output)?,
        Value::Object(serde_json::Map::default())
    );
    Ok(())
}
#[test]
fn invalid_payloads_use_exit_two_without_echoing_input()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let private = "ghp_abcdefghijklmnopqrstuvwxyz1234567890";
    let cases = [
        Vec::new(),
        format!("{{\"prompt\":\"{private}\"").into_bytes(),
        payload("Stop", private, fixture.repository())?,
        serde_json::to_vec(
            &serde_json :: json ! ({ "hook_event_name" : "UserPromptSubmit" , "cwd" : fixture . repository () , }),
        )?,
    ];
    for bytes in cases {
        let output = fixture.run(["hook", "--agent", "claude"], &bytes)?;
        assert_exit(&output, 2);
        assert!(output.stdout.is_empty());
        assert!(!String::from_utf8_lossy(&output.stderr).contains(private));
    }
    let oversized = vec![b'a'; 1024 * 1024 + 1];
    assert_error(
        &fixture.run(["hook", "--agent", "codex"], &oversized)?,
        2,
        "input_too_large",
    )?;
    Ok(())
}
#[test]
fn an_unavailable_project_git_process_exits_four_without_context()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable hook architecture",
        fixture.repository(),
    )?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
        .args(["hook", "--agent", "codex"])
        .current_dir(fixture.repository())
        .env("AGENT_MEMORY_ROOT", fixture.root())
        .env("PATH", fixture.root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    std::io::Write::write_all(
        child.stdin.as_mut().ok_or("child stdin is not piped")?,
        &payload,
    )?;
    let output = child.wait_with_output()?;
    assert_error(&output, 4, "scope_unavailable")?;
    Ok(())
}
#[test]
fn cursor_is_not_a_native_hook_variant() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let output = fixture.run(["hook", "--agent", "cursor"], b"{}")?;
    assert_error(&output, 2, "invalid_arguments")?;
    Ok(())
}
#[test]
fn an_unavailable_store_never_reuses_prior_context()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let statement = "Durable unavailable store memory.";
    let draft = CliFixture::git_draft("invariant", statement, "durable unavailable store")?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft)?, 0);
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable unavailable store",
        fixture.repository(),
    )?;
    assert_exit(&fixture.run(["hook", "--agent", "codex"], &payload)?, 0);
    fs::set_permissions(fixture.root(), fs::Permissions::from_mode(0o755))?;
    let output = fixture.run(["hook", "--agent", "codex"], &payload)?;
    assert_exit(&output, 4);
    assert!(output.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&output.stderr).contains(statement));
    Ok(())
}
#[test]
fn an_unavailable_selected_yaml_exits_four_without_context()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let statement = "Durable selected path memory.";
    let draft = CliFixture::git_draft("invariant", statement, "durable selected path")?;
    let admitted = fixture.run(["admit", "--format", "json"], &draft)?;
    assert_exit(&admitted, 0);
    let id = (stdout_json(&admitted)?
        .get("id")
        .ok_or("missing admitted memory ID")?)
    .as_str()
    .ok_or("expected JSON string")?
    .to_owned();
    let path =
        find_entry(fixture.root(), &format!("{id}.yaml"))?.ok_or("stored YAML entry missing")?;
    let displaced = fixture.root().join("displaced-selected.yaml");
    fs::rename(&path, &displaced)?;
    symlink(&displaced, &path)?;
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable selected path",
        fixture.repository(),
    )?;
    let output = fixture.run(["hook", "--agent", "claude"], &payload)?;
    assert_error(&output, 4, "unsafe_store_path")?;
    assert!(!String::from_utf8_lossy(&output.stderr).contains(statement));
    Ok(())
}
#[test]
fn a_hook_stdout_failure_uses_exit_four_without_context_on_stderr()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let statement = "Durable stdout memory.";
    let draft = CliFixture::git_draft("invariant", statement, "durable stdout memory")?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft)?, 0);
    let payload = payload(
        "UserPromptSubmit",
        "Apply durable stdout memory",
        fixture.repository(),
    )?;
    let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
        .args(["hook", "--agent", "claude"])
        .current_dir(fixture.repository())
        .env("AGENT_MEMORY_ROOT", fixture.root())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    drop(child.stdout.take());
    std::io::Write::write_all(
        child.stdin.as_mut().ok_or("child stdin is not piped")?,
        &payload,
    )?;
    let output = child.wait_with_output()?;
    assert_exit(&output, 4);
    let diagnostic: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(
        (*diagnostic
            .pointer("/error/code")
            .ok_or("missing hook response /error/code")?),
        "output_unavailable"
    );
    assert!(!String::from_utf8_lossy(&output.stderr).contains(statement));
    Ok(())
}
fn payload(
    event: &str,
    prompt: &str,
    cwd: &std::path::Path,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::to_vec(
        &serde_json :: json ! ({ "session_id" : "session-fixture" , "transcript_path" : "/private/transcript.jsonl" , "cwd" : cwd , "permission_mode" : "default" , "hook_event_name" : event , "prompt" : prompt , }),
    )?)
}
fn find_entry(
    directory: &std::path::Path,
    name: &str,
) -> Result<Option<std::path::PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.is_dir() {
            if let Some(found) = find_entry(&path, name)? {
                return Ok(Some(found));
            }
        } else if path.file_name().and_then(|value| value.to_str()) == Some(name) {
            return Ok(Some(path));
        }
    }
    Ok(None)
}
