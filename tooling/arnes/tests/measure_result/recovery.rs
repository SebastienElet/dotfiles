use super::measure_support::*;
use serde_json::{Value, json};
use std::fs;
fn seed_result_one_revision_behind(
    harness: &Harness,
    run_id: &str,
    result_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_success(&harness.run(&[
        "measure",
        "finish",
        run_id,
        "--merge-ready",
        "fail",
        "--human-minutes",
        "1",
        "--failure-reason",
        "revision one",
    ])?);
    let revision_one = fs::read(result_path)?;
    assert_success(&harness.run(&[
        "measure",
        "finish",
        run_id,
        "--merge-ready",
        "unjudgeable",
        "--human-minutes",
        "2",
        "--evidence",
        "revision two evidence",
    ])?);
    fs::write(result_path, revision_one)?;
    Ok(())
}
#[test]
fn finish_recovers_a_missing_result_from_the_complete_event_snapshot()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let run = harness.run_path(&run_id);
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "fail",
        "--human-minutes",
        "3",
        "--failure-reason",
        "first adjudication",
        "--evidence",
        "review-one",
    ])?);
    fs::remove_file(run.join("result.json"))?;
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "5",
        "--evidence",
        "review-two",
    ])?);
    let result = read_json(run.join("result.json"))?;
    assert_eq!(
        *(result)
            .get("revision")
            .ok_or("missing fixture index revision")?,
        2
    );
    assert_eq!(
        *(result)
            .get("merge_ready")
            .ok_or("missing fixture index merge_ready")?,
        "pass"
    );
    let events = read_jsonl(run.join("events.jsonl"))?;
    let decisions: Vec<&Value> = events
        .iter()
        .map(
            |event| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(event).get("event").ok_or("missing fixture index event")? == "result_recorded"
                };
                Ok((include, event))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .collect();
    assert_eq!(decisions.len(), 2);
    assert_eq!(
        *(*(*(decisions).first().ok_or("missing fixture index 0")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("revision")
        .ok_or("missing fixture index revision")?,
        1
    );
    assert_eq!(
        *(*(*(decisions).first().ok_or("missing fixture index 0")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("merge_ready")
        .ok_or("missing fixture index merge_ready")?,
        "fail"
    );
    assert_eq!(
        *(*(*(decisions).first().ok_or("missing fixture index 0")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("failure_reason")
        .ok_or("missing fixture index failure_reason")?,
        "first adjudication"
    );
    assert_eq!(
        *(*(*(decisions).get(1).ok_or("missing fixture index 1")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("revision")
        .ok_or("missing fixture index revision")?,
        2
    );
    assert_eq!(
        *(*(*(decisions).get(1).ok_or("missing fixture index 1")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("merge_ready")
        .ok_or("missing fixture index merge_ready")?,
        "pass"
    );
    Ok(())
}
#[test]
fn finish_recovers_when_result_is_exactly_one_revision_behind_history()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    seed_result_one_revision_behind(&harness, &run_id, &result_path)?;
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "3",
    ])?);
    let result = read_json(&result_path)?;
    assert_eq!(
        *(result)
            .get("revision")
            .ok_or("missing fixture index revision")?,
        3
    );
    assert_eq!(
        *(result)
            .get("merge_ready")
            .ok_or("missing fixture index merge_ready")?,
        "pass"
    );
    let events = read_jsonl(run.join("events.jsonl"))?;
    let decisions: Vec<&Value> = events
        .iter()
        .map(
            |event| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(event).get("event").ok_or("missing fixture index event")? == "result_recorded"
                };
                Ok((include, event))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .collect();
    assert_eq!(decisions.len(), 3);
    assert_eq!(
        *(*(*(decisions).first().ok_or("missing fixture index 0")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("failure_reason")
        .ok_or("missing fixture index failure_reason")?,
        "revision one"
    );
    assert_eq!(
        *(*(*(decisions).get(1).ok_or("missing fixture index 1")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("merge_ready")
        .ok_or("missing fixture index merge_ready")?,
        "unjudgeable"
    );
    assert_eq!(
        *(*(*(*(decisions).get(1).ok_or("missing fixture index 1")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("evidence")
        .ok_or("missing fixture index evidence")?)
        .get(0)
        .ok_or("missing fixture index 0")?,
        "revision two evidence"
    );
    assert_eq!(
        *(*(*(decisions).get(2).ok_or("missing fixture index 2")?)
            .get("result")
            .ok_or("missing fixture index result")?)
        .get("merge_ready")
        .ok_or("missing fixture index merge_ready")?,
        "pass"
    );
    Ok(())
}
#[test]
fn finish_refuses_a_result_more_than_one_revision_behind_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    let mut revision_one = None;
    for minutes in 1..=3 {
        assert_success(&harness.run(&[
            "measure",
            "finish",
            &run_id,
            "--merge-ready",
            "pass",
            "--human-minutes",
            &minutes.to_string(),
        ])?);
        if minutes == 1 {
            revision_one = Some(fs::read(&result_path)?);
        }
    }
    fs::write(
        &result_path,
        revision_one.ok_or("required test value is missing")?,
    )?;
    let result_before = fs::read(&result_path)?;
    let events_path = run.join("events.jsonl");
    let events_before = fs::read(&events_path)?;
    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "4",
    ])?;
    assert_failure(&output, "result.json diverges from result_recorded history");
    assert_eq!(fs::read(result_path)?, result_before);
    assert_eq!(fs::read(events_path)?, events_before);
    Ok(())
}
#[test]
fn finish_refuses_result_and_event_history_divergence_without_mutation()
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
    let result_path = run.join("result.json");
    let events_path = run.join("events.jsonl");
    let mut result = read_json(&result_path)?;
    {
        result
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("human_minutes").to_owned(), json!(999.0));
    };
    fs::write(&result_path, serde_json::to_vec(&result)?)?;
    let result_before = fs::read(&result_path)?;
    let events_before = fs::read(&events_path)?;
    let output = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "2",
    ])?;
    assert_failure(&output, "result.json diverges from result_recorded history");
    assert_eq!(fs::read(result_path)?, result_before);
    assert_eq!(fs::read(events_path)?, events_before);
    Ok(())
}
#[test]
fn list_refuses_a_result_more_than_one_revision_behind_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let run = harness.run_path(&run_id);
    let result_path = run.join("result.json");
    let mut revision_one = None;
    for minutes in 1..=3 {
        assert_success(&harness.run(&[
            "measure",
            "finish",
            &run_id,
            "--merge-ready",
            "pass",
            "--human-minutes",
            &minutes.to_string(),
        ])?);
        if minutes == 1 {
            revision_one = Some(fs::read(&result_path)?);
        }
    }
    fs::write(
        &result_path,
        revision_one.ok_or("required test value is missing")?,
    )?;
    let result_before = fs::read(&result_path)?;
    let events_path = run.join("events.jsonl");
    let events_before = fs::read(&events_path)?;
    let output = harness.run(&["measure", "list", "--format", "json"])?;
    assert_failure(&output, "result.json diverges from result_recorded history");
    assert_eq!(fs::read(result_path)?, result_before);
    assert_eq!(fs::read(events_path)?, events_before);
    Ok(())
}
