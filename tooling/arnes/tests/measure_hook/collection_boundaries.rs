use super::support::*;
#[test]
fn refuses_directory_and_file_symlink_attacks()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let outside = tempfile::tempdir()?;
    symlink(outside.path(), harness.state.join("dotfiles"))?;
    let output = harness.run(
        "codex",
        br#"{"session_id":"session","event":"SessionStart"}"#,
    )?;
    assert_advisory_failure(&output);
    assert!(fs::read_dir(outside.path())?.next().is_none());
    fs::remove_file(harness.state.join("dotfiles"))?;
    assert_success(&harness.run(
        "codex",
        br#"{"session_id":"session","event":"SessionStart"}"#,
    )?);
    let events = harness.only_run()?.join("events.jsonl");
    fs::remove_file(&events)?;
    let captured = outside.path().join("captured");
    fs::write(&captured, "sentinel")?;
    symlink(&captured, &events)?;
    let output = harness.run("codex", br#"{"session_id":"session","event":"Followup"}"#)?;
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(captured)?, "sentinel");
    Ok(())
}
#[test]
fn refuses_hardlinked_and_corrupt_managed_files()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    assert_success(&harness.run("codex", payload)?);
    let run = harness.only_run()?;
    let run_json = run.join("run.json");
    let original_run = fs::read(&run_json)?;
    fs::write(&run_json, "null")?;
    let output = harness.run("codex", payload)?;
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(&run_json)?, "null");
    fs::write(&run_json, original_run)?;
    let events = run.join("events.jsonl");
    fs::remove_file(&events)?;
    let outside = harness.repository.join("tracked");
    fs::write(&outside, "sentinel")?;
    fs::hard_link(&outside, &events)?;
    let output = harness.run("codex", payload)?;
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(&outside)?, "sentinel");
    fs::remove_file(&events)?;
    fs::write(&events, r#"{"partial"#)?;
    let output = harness.run("codex", payload)?;
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(events)?, r#"{"partial"#);
    Ok(())
}
#[test]
fn refuses_incompletely_typed_run_records_without_appending()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (field, replacement) in [
        ("model_fingerprint", Some(json!([]))),
        ("repository_commit", Some(json!(["wrong"]))),
        ("model_fingerprint", None),
        ("harness_fingerprint_limitations", Some(json!(false))),
    ] {
        let harness = Harness::new()?;
        let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
        assert_success(&harness.run("codex", payload)?);
        let run = harness.only_run()?;
        let run_json = run.join("run.json");
        let events = run.join("events.jsonl");
        let before_events = fs::read(&events)?;
        let mut record = read_json(&run_json)?;
        match replacement {
            Some(value) => {
                record
                    .as_object_mut()
                    .ok_or("expected hook payload object")?
                    .insert(field.to_owned(), value);
            }
            None => {
                record
                    .as_object_mut()
                    .ok_or("expected JSON object")?
                    .remove(field);
            }
        }
        let corrupted = serde_json::to_vec(&record)?;
        fs::write(&run_json, &corrupted)?;
        assert_eq!(harness.runs()?.len(), 1);
        assert_eq!(fs::read(&run_json)?, corrupted);
        let output = harness.run("codex", payload)?;
        assert_eq!(harness.runs()?, vec![run.clone()]);
        assert_eq!(fs::read(&run_json)?, corrupted, "field {field}");
        assert_advisory_failure(&output);
        assert_eq!(fs::read(&events)?, before_events, "field {field}");
    };
    Ok(())
}
#[test]
fn refuses_duplicate_run_record_keys_without_appending()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    assert_success(&harness.run("codex", payload)?);
    let run = harness.only_run()?;
    let run_json = run.join("run.json");
    let events = run.join("events.jsonl");
    let original = fs::read_to_string(&run_json)?;
    let corrupted = original.replacen('{', r#"{"agent":"evil","#, 1);
    fs::write(&run_json, &corrupted)?;
    let before_events = fs::read(&events)?;
    let output = harness.run("codex", payload)?;
    assert_advisory_failure(&output);
    assert_eq!(fs::read_to_string(run_json)?, corrupted);
    assert_eq!(fs::read(events)?, before_events);
    Ok(())
}
#[test]
fn refuses_a_broken_run_json_symlink_instead_of_replacing_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    assert_success(&harness.run("codex", payload)?);
    let run_json = harness.only_run()?.join("run.json");
    fs::remove_file(&run_json)?;
    symlink("missing-target", &run_json)?;
    let output = harness.run("codex", payload)?;
    assert_advisory_failure(&output);
    assert_eq!(fs::read_link(run_json)?, Path::new("missing-target"));
    Ok(())
}
#[path = "collection_boundaries/advisory.rs"]
mod advisory;
#[path = "collection_boundaries/redaction.rs"]
mod redaction;
