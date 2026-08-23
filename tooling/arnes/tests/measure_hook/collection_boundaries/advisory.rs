use super::super::support::*;

#[test]
fn collection_failure_after_an_existing_run_is_advisory_and_journaled() {
    let harness = Harness::new();
    assert_success(&harness.run("claude-code", br#"{"session_id":"existing"}"#));
    fs::create_dir_all(harness.home.join(".claude")).unwrap();
    fs::write(harness.home.join(".claude/settings.json"), "{").unwrap();
    let payload = br#"{"session_id":"new","event":"SessionStart"}"#;

    let output = harness.run("claude-code", payload);

    assert_advisory_failure(&output);
    assert!(!output.stderr.is_empty());
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
