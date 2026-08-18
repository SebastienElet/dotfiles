use super::*;

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
fn any_thought_discriminator_neutralizes_contradictory_payloads() {
    for payload in [
        json!({
            "session_id":"session",
            "hook_event_name":"afterAgentThought",
            "event":"afterAgentResponse",
            "type":"afterAgentResponse",
            "text":"private-thought"
        }),
        json!({
            "session_id":"session",
            "hook_event_name":"afterAgentResponse",
            "event":"afterAgentThought",
            "type":"afterAgentResponse",
            "text":"private-thought"
        }),
        json!({
            "session_id":"session",
            "hook_event_name":"afterAgentResponse",
            "event":"afterAgentResponse",
            "type":"afterAgentThought",
            "text":"private-thought"
        }),
    ] {
        let harness = Harness::new();
        assert_success(&harness.run("claude-code", payload.to_string().as_bytes()));
        let stored = walk(&harness.only_run())
            .into_iter()
            .filter(|path| path.is_file())
            .flat_map(|path| fs::read(path).unwrap())
            .collect::<Vec<_>>();
        assert!(!String::from_utf8_lossy(&stored).contains("private-thought"));
    }
}

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
