use super::support::*;

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
        ("model_fingerprint", Some(json!([]))),
        ("repository_commit", Some(json!(["wrong"]))),
        ("model_fingerprint", None),
        ("harness_fingerprint_limitations", Some(json!(false))),
    ] {
        let harness = Harness::new();
        let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
        assert_success(&harness.run("codex", payload));
        let run = harness.only_run();
        let run_json = run.join("run.json");
        let events = run.join("events.jsonl");
        let before_events = fs::read(&events).unwrap();
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
    let original = fs::read_to_string(&run_json).unwrap();
    let corrupted = original.replacen('{', r#"{"agent":"evil","#, 1);
    fs::write(&run_json, &corrupted).unwrap();
    let before_events = fs::read(&events).unwrap();

    let output = harness.run("codex", payload);

    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(run_json).unwrap(), corrupted);
    assert_eq!(fs::read(events).unwrap(), before_events);
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

#[path = "collection_boundaries/advisory.rs"]
mod advisory;
#[path = "collection_boundaries/redaction.rs"]
mod redaction;
