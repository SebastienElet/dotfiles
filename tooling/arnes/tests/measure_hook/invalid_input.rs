use super::support::*;

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
