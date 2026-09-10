use super::measure_support::*;
use serde_json::json;
use std::fs;
fn record_private_feedback(
    harness: &Harness,
    run_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_success(&harness.run(&[
        "measure",
        "feedback",
        run_id,
        "--source-type",
        "human",
        "--source-id",
        "reviewer@example.test",
        "--scope",
        "diff",
        "--observed",
        "private line one\nprivate line two",
        "--expected",
        "handle the empty response",
        "--evidence",
        "review-thread-1",
        "--invariant",
        "empty-response-safe",
        "--severity",
        "blocking",
        "--adjudication",
        "confirmed",
        "--resolution",
        "open",
        "--failure-category",
        "correctness",
    ])?);
    Ok(())
}
#[test]
fn feedback_preserves_private_multiline_text_and_never_mutates_result()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("claude-code", "session_id", "session", "prompt")?;
    assert_success(&harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "1",
    ])?);
    let result_before = fs::read(harness.run_path(&run_id).join("result.json"))?;
    record_private_feedback(&harness, &run_id)?;
    let feedback = read_jsonl(harness.run_path(&run_id).join("feedback.jsonl"))?;
    assert_eq!(feedback.len(), 1);
    assert_eq!(
        *(*(feedback).first().ok_or("missing fixture index 0")?)
            .get("source_type")
            .ok_or("missing fixture index source_type")?,
        "human"
    );
    assert_eq!(
        *(*(feedback).first().ok_or("missing fixture index 0")?)
            .get("observed")
            .ok_or("missing fixture index observed")?,
        "private line one\nprivate line two"
    );
    assert_eq!(
        *(*(feedback).first().ok_or("missing fixture index 0")?)
            .get("analysis_blocking")
            .ok_or("missing fixture index analysis_blocking")?,
        true
    );
    assert_eq!(
        fs::read(harness.run_path(&run_id).join("result.json"))?,
        result_before
    );
    Ok(())
}
#[test]
fn only_confirmed_blocking_feedback_is_marked_for_analysis()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("cursor", "conversation_id", "session", "prompt")?;
    let _: () = for (severity, adjudication, expected) in [
        ("blocking", "pending", false),
        ("major", "confirmed", false),
        ("blocking", "confirmed", true),
    ] {
        let output = harness.run(&[
            "measure",
            "feedback",
            &run_id,
            "--source-type",
            "harness",
            "--source-id",
            "ci-review",
            "--scope",
            "change",
            "--observed",
            "behavior",
            "--expected",
            "other behavior",
            "--severity",
            severity,
            "--adjudication",
            adjudication,
            "--resolution",
            "open",
            "--failure-category",
            "requirements",
        ])?;
        assert_success(&output);
        let feedback = read_jsonl(harness.run_path(&run_id).join("feedback.jsonl"))?;
        assert_eq!(
            *(feedback.last().ok_or("required test value is missing")?)
                .get("analysis_blocking")
                .ok_or("missing fixture index analysis_blocking")?,
            expected
        );
    };
    Ok(())
}
#[test]
fn feedback_rejects_a_semantically_invalid_existing_log()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let path = harness.run_path(&run_id).join("feedback.jsonl");
    fs::write(&path, b"{\"schema_version\":1}\n")?;
    let output = harness.run(&[
        "measure",
        "feedback",
        &run_id,
        "--source-type",
        "human",
        "--source-id",
        "reviewer",
        "--scope",
        "diff",
        "--observed",
        "observation",
        "--expected",
        "expectation",
        "--severity",
        "minor",
        "--adjudication",
        "pending",
        "--resolution",
        "open",
        "--failure-category",
        "maintainability",
    ])?;
    assert_failure(&output, "feedback.jsonl has an invalid record");
    assert_eq!(fs::read_to_string(path)?.lines().count(), 1);
    Ok(())
}
#[test]
fn feedback_rejects_duplicate_fields_in_existing_records()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let path = harness.run_path(&run_id).join("feedback.jsonl");
    let record = json ! ({ "schema_version" : 1 , "feedback_id" : "a" . repeat (64) , "run_id" : run_id , "recorded_at_ms" : 1 , "source_type" : "human" , "source_id" : "first" , "scope" : "diff" , "observed" : "observation" , "expected" : "expectation" , "evidence" : [] , "invariants" : [] , "severity" : "minor" , "adjudication" : "pending" , "resolution" : "open" , "failure_category" : "maintainability" , "analysis_blocking" : false });
    let line = serde_json::to_string(&record)?.replace(
        "\"source_id\":\"first\"",
        "\"source_id\":\"first\",\"source_id\":\"second\"",
    );
    fs::write(&path, format!("{line}\n"))?;
    let output = harness.run(&[
        "measure",
        "feedback",
        (*(record)
            .get("run_id")
            .ok_or("missing fixture index run_id")?)
        .as_str()
        .ok_or("expected JSON string")?,
        "--source-type",
        "human",
        "--source-id",
        "reviewer",
        "--scope",
        "diff",
        "--observed",
        "observation",
        "--expected",
        "expectation",
        "--severity",
        "minor",
        "--adjudication",
        "pending",
        "--resolution",
        "open",
        "--failure-category",
        "maintainability",
    ])?;
    assert_failure(&output, "feedback.jsonl has an invalid record");
    Ok(())
}
#[test]
fn parallel_feedback_appends_are_complete_and_unique()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let mut children = Vec::new();
    for index in 0..16 {
        children.push(
            harness
                .command()
                .args([
                    "measure",
                    "feedback",
                    &run_id,
                    "--source-type",
                    "human",
                    "--source-id",
                    &format!("reviewer-{index}"),
                    "--scope",
                    "diff",
                    "--observed",
                    "observation",
                    "--expected",
                    "expectation",
                    "--severity",
                    "minor",
                    "--adjudication",
                    "pending",
                    "--resolution",
                    "open",
                    "--failure-category",
                    "maintainability",
                ])
                .spawn()?,
        );
    }
    for child in children {
        assert_success(&child.wait_with_output()?);
    }
    let feedback = read_jsonl(harness.run_path(&run_id).join("feedback.jsonl"))?;
    assert_eq!(feedback.len(), 16);
    let mut ids: Vec<&str> = feedback
        .iter()
        .map(
            |item| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok((*(item)
                    .get("source_id")
                    .ok_or("missing fixture index source_id")?)
                .as_str()
                .ok_or("expected JSON string")?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 16);
    Ok(())
}
