use super::measure_support::*;
use serde_json::{Value, json};
use std::fs;
#[test]
fn lists_pending_runs_without_private_context_and_filters_by_agent()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let codex = harness.capture(
        "codex",
        "session_id",
        "codex-session",
        "  first\n prompt with enough detail  ",
    )?;
    harness.capture(
        "cursor",
        "conversation_id",
        "cursor-session",
        "cursor prompt",
    )?;
    let output = harness.run(&["measure", "list", "--agent", "codex", "--format", "json"])?;
    assert_success(&output);
    let runs: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(runs.as_array().ok_or("expected JSON array")?.len(), 1);
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("run_id")
            .ok_or("missing fixture index run_id")?,
        codex
    );
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("agent")
            .ok_or("missing fixture index agent")?,
        "codex"
    );
    assert!(
        (*(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("repository")
            .ok_or("missing fixture index repository")?)
        .is_null()
    );
    assert!(
        (*(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("first_prompt_excerpt")
            .ok_or("missing fixture index first_prompt_excerpt")?)
        .is_null()
    );
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("last_event")
            .ok_or("missing fixture index last_event")?,
        "prompt.submit"
    );
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("has_result")
            .ok_or("missing fixture index has_result")?,
        false
    );
    Ok(())
}
#[test]
fn list_rejects_prompt_records_missing_contract_fields()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    fs::write(
        harness.run_path(&run_id).join("prompts.jsonl"),
        b"{\"prompt\":\"fake prompt\"}\n",
    )?;
    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"])?,
        "prompts.jsonl has an invalid record",
    );
    Ok(())
}
#[test]
fn list_rejects_duplicate_prompt_fields() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    fs :: write (harness . run_path (& run_id) . join ("prompts.jsonl") , b"{\"timestamp_ms\":1,\"event_id\":\"event\",\"session_id\":\"session\",\"prompt_id\":null,\"prompt\":\"first\",\"prompt\":\"second\"}\n" ,) ? ;
    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"])?,
        "prompts.jsonl has an invalid record",
    );
    Ok(())
}
#[test]
fn list_rejects_event_records_with_wrong_field_types()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    fs::write(
        harness.run_path(&run_id).join("events.jsonl"),
        b"{\"event\":\"prompt.submit\",\"timestamp_ms\":\"invalid\"}\n",
    )?;
    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"])?,
        "events.jsonl has an invalid record",
    );
    Ok(())
}
#[test]
fn list_rejects_duplicate_event_fields() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let path = harness.run_path(&run_id).join("events.jsonl");
    let content = fs::read_to_string(&path)?.replace(
        "\"event\":\"prompt.submit\"",
        "\"event\":\"prompt.submit\",\"event\":\"agent.stop\"",
    );
    fs::write(path, content)?;
    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"])?,
        "events.jsonl has an invalid record",
    );
    Ok(())
}
#[test]
fn list_rejects_an_event_log_without_a_terminal_newline()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let path = harness.run_path(&run_id).join("events.jsonl");
    let mut content = fs::read(&path)?;
    assert_eq!(content.pop(), Some(b'\n'));
    fs::write(path, content)?;
    assert_failure(
        &harness.run(&["measure", "list", "--format", "json"])?,
        "events.jsonl is truncated or oversized",
    );
    Ok(())
}
#[test]
fn human_list_marks_a_finalized_run_without_inference_from_agent_stop()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("claude-code", "session_id", "session", "implement it")?;
    let before = harness.run(&["measure", "list"])?;
    assert_success(&before);
    let before = String::from_utf8(before.stdout)?;
    assert!(before.contains(&run_id));
    assert!(before.contains("result=pending"));
    let finish = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "2.5",
    ])?;
    assert_success(&finish);
    let after = String::from_utf8(harness.run(&["measure", "list"])?.stdout)?;
    assert!(after.contains("result=recorded"));
    Ok(())
}
#[test]
fn list_reports_a_missing_result_snapshot_without_repairing_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let run = harness.run_path(&run_id);
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ])?);
    let events_before = fs::read(run.join("events.jsonl"))?;
    fs::remove_file(run.join("result.json"))?;
    let output = harness.run(&["measure", "list", "--format", "json"])?;
    assert_success(&output);
    let runs: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("has_result")
            .ok_or("missing fixture index has_result")?,
        false
    );
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("result_state")
            .ok_or("missing fixture index result_state")?,
        "missing"
    );
    assert!(!run.join("result.json").exists());
    assert_eq!(fs::read(run.join("events.jsonl"))?, events_before);
    Ok(())
}
#[test]
fn list_reports_a_one_revision_lag_without_repairing_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ])?);
    let revision_one = fs::read(&result_path)?;
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "2",
    ])?);
    fs::write(&result_path, revision_one)?;
    let result_before = fs::read(&result_path)?;
    let events_before = fs::read(run.join("events.jsonl"))?;
    let output = harness.run(&["measure", "list", "--format", "json"])?;
    assert_success(&output);
    let runs: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("has_result")
            .ok_or("missing fixture index has_result")?,
        true
    );
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("result_state")
            .ok_or("missing fixture index result_state")?,
        "lagging"
    );
    assert_eq!(fs::read(result_path)?, result_before);
    assert_eq!(fs::read(run.join("events.jsonl"))?, events_before);
    Ok(())
}
#[test]
fn human_list_escapes_terminal_controls_without_changing_json()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let agent = "codex\u{1b}]52;c;dGVzdA==\u{7}\u{9d}\u{7f}";
    let run_id = harness.capture("codex", "session_id", "session", "private prompt")?;
    let run_path = harness.run_path(&run_id);
    let run_json = run_path.join("run.json");
    let mut run = read_json(&run_json)?;
    {
        run.as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("schema_version").to_owned(), json!(1));
    };
    {
        run.as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("session_id").to_owned(), json!("session"));
    };
    {
        run.as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("repository").to_owned(), json!(agent));
    };
    {
        run.as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("repository_branch").to_owned(), Value::Null);
    };
    {
        run.as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("model").to_owned(), Value::Null);
    };
    run.as_object_mut()
        .ok_or("expected JSON object")?
        .remove("model_fingerprint");
    run.as_object_mut()
        .ok_or("expected JSON object")?
        .remove("operating_system");
    run.as_object_mut()
        .ok_or("expected JSON object")?
        .remove("architecture");
    fs::write(&run_json, serde_json::to_vec(&run)?)?;
    let prompt = json ! ({ "timestamp_ms" : (run.get("started_at_ms").ok_or("missing fixture timestamp")?) , "event_id" : "a" . repeat (64) , "session_id" : "session" , "prompt_id" : Value :: Null , "prompt" : agent });
    fs::write(
        run_path.join("prompts.jsonl"),
        format!("{}\n", serde_json::to_string(&prompt)?),
    )?;
    let human = harness.run(&["measure", "list"])?;
    assert_success(&human);
    let human = String::from_utf8(human.stdout)?;
    assert!(
        human
            .chars()
            .all(|character| character == '\n' || !character.is_control())
    );
    assert!(human.contains("\\u{001b}]52"));
    assert!(human.contains("\\u{009d}"));
    assert!(human.contains("\\u{007f}"));
    let json_output = harness.run(&["measure", "list", "--format", "json"])?;
    assert_success(&json_output);
    let runs: Value = serde_json::from_slice(&json_output.stdout)?;
    assert_eq!(
        *(*(runs).get(0).ok_or("missing fixture index 0")?)
            .get("repository")
            .ok_or("missing fixture index repository")?,
        agent
    );
    Ok(())
}
