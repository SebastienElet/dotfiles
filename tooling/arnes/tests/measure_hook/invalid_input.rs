use super::support::*;
#[test]
fn rejects_missing_session_without_persisting_the_payload()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let secret = "missing-session-secret";
    let output = harness.run(
        "codex",
        json ! ({ "event" : "SessionStart" , "value" : secret })
            .to_string()
            .as_bytes(),
    )?;
    assert_advisory_failure(&output);
    assert!(String::from_utf8(output.stderr)?.contains("session_id"));
    assert!(harness.runs()?.is_empty());
    let invalid = fs::read_to_string(harness.measure_root().join("invalid.jsonl"))?;
    assert!(!invalid.contains(secret));
    Ok(())
}
#[test]
fn invalid_and_oversized_json_store_only_safe_metadata()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for payload in [b"not-json".to_vec(), vec![b'x'; 1_048_577]] {
        let harness = Harness::new()?;
        let output = harness.run("codex", &payload)?;
        assert_advisory_failure(&output);
        let records = read_jsonl(harness.measure_root().join("invalid.jsonl"))?;
        assert_eq!(records.len(), 1);
        assert_eq!(
            *(*(records).first().ok_or("missing fixture index 0")?)
                .get("agent")
                .ok_or("missing fixture index agent")?,
            "codex"
        );
        assert_eq!(
            *(*(records).first().ok_or("missing fixture index 0")?)
                .get("size")
                .ok_or("missing fixture index size")?,
            payload.len()
        );
        assert_eq!(
            (*(*(records).first().ok_or("missing fixture index 0")?)
                .get("sha256")
                .ok_or("missing fixture index sha256")?)
            .as_str()
            .ok_or("expected JSON string")?
            .len(),
            64
        );
        assert!(
            (*(records).first().ok_or("missing fixture index 0")?)
                .get("payload")
                .is_none()
        );
        assert!(harness.runs()?.is_empty());
    };
    Ok(())
}
#[test]
fn falls_back_to_home_local_state_when_xdg_state_is_absent()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let mut command = harness.command("codex");
    command.env_remove("XDG_STATE_HOME");
    let mut child = command.spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(br#"{"session_id":"session","event":"SessionStart"}"#)?;
    let output = child.wait_with_output()?;
    assert_success(&output);
    assert!(
        harness
            .home
            .join(".local/state/dotfiles/agent-harness/runs")
            .is_dir()
    );
    Ok(())
}
#[test]
fn home_without_git_can_use_default_state_below_home()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let mut command = harness.command("codex");
    command
        .current_dir(&harness.home)
        .env_remove("XDG_STATE_HOME");
    let mut child = command.spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(br#"{"session_id":"session"}"#)?;
    assert_success(&child.wait_with_output()?);
    assert!(
        harness
            .home
            .join(".local/state/dotfiles/agent-harness/runs")
            .is_dir()
    );
    Ok(())
}
#[test]
fn recursively_duplicate_json_keys_are_advisory_and_never_create_a_run()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = br#"{"session_id":"session","nested":{"value":1,"value":2}}"#;
    let output = harness.run("codex", payload)?;
    assert_advisory_failure(&output);
    assert!(harness.runs()?.is_empty());
    let invalid = read_jsonl(harness.measure_root().join("invalid.jsonl"))?;
    assert_eq!(invalid.len(), 1);
    assert_eq!(
        *(*(invalid).first().ok_or("missing fixture index 0")?)
            .get("size")
            .ok_or("missing fixture index size")?,
        payload.len()
    );
    assert!(
        (*(*(invalid).first().ok_or("missing fixture index 0")?)
            .get("error")
            .ok_or("missing fixture index error")?)
        .as_str()
        .ok_or("expected JSON string")?
        .contains("duplicate")
    );
    Ok(())
}
