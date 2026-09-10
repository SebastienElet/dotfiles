#![cfg(test)]
#[path = "measure_result/hook_fixture.rs"]
mod hook_fixture;
#[path = "measure_result/support.rs"]
mod measure_support;
use measure_support::*;
use serde_json::{Value, json};
use std::fs;
#[test]
fn reports_judgeability_success_activity_latency_and_volume_by_agent()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new_v2()?;
    let passing = harness.capture("codex", "session_id", "passing", "fixture prompt")?;
    assert_success(&harness.hook(
        "codex",
        &json ! ({ "session_id" : "passing" , "hook_event_name" : "PreToolUse" }),
    )?);
    let failing = harness.capture("claude-code", "session_id", "failing", "fixture prompt")?;
    let unjudgeable =
        harness.capture("cursor", "conversation_id", "unjudgeable", "fixture prompt")?;
    harness.capture("codex", "session_id", "pending", "fixture prompt")?;
    set_times(&harness, &passing, 100, &[200, 300])?;
    set_times(&harness, &failing, 400, &[500])?;
    set_times(&harness, &unjudgeable, 600, &[700])?;
    record_outcome(&harness, &passing, "pass", &["--oracle", "cargo-test"])?;
    record_outcome(&harness, &failing, "fail", &["--oracle", "cargo-test"])?;
    record_outcome(
        &harness,
        &unjudgeable,
        "unjudgeable",
        &["--reason", "missing-oracle"],
    )?;
    let output = harness.run(&["measure", "report", "--format", "json"])?;
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout)?;
    assert!(harness.state_root().is_dir());
    let totals = report.get("totals").ok_or("missing report totals")?;
    for (field, expected) in [
        ("runs", json!(4)),
        ("judgeable_runs", json!(2)),
        ("judgeable_rate", json!(0.5)),
        ("declared_successful_runs", json!(1)),
        ("declared_success_rate", json!(0.5)),
        ("event_count", json!(5)),
        ("tool_call_count", json!(1)),
        ("latency_runs", json!(4)),
    ] {
        assert_eq!(totals.get(field), Some(&expected), "{field}");
    }
    let storage = report.get("storage").ok_or("missing report storage")?;
    for field in ["logical_bytes", "allocated_bytes"] {
        let bytes = storage
            .get(field)
            .and_then(Value::as_u64)
            .ok_or("missing storage byte count")?;
        assert!(bytes > 0, "{field}");
    }
    let agents = report
        .get("agents")
        .and_then(Value::as_array)
        .ok_or("missing report agents")?;
    for (index, expected) in ["codex", "claude-code", "cursor"].into_iter().enumerate() {
        let agent = agents.get(index).ok_or("missing expected agent")?;
        assert_eq!(agent.get("agent"), Some(&json!(expected)));
    }
    let codex = agents.first().ok_or("missing Codex report")?;
    assert_eq!(codex.pointer("/metrics/runs"), Some(&json!(2)));

    Ok(())
}
#[test]
fn rejects_invalid_event_data_instead_of_omitting_the_run()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new_v2()?;
    let run_id = harness.capture("codex", "session_id", "invalid", "fixture prompt")?;
    fs::write(
        harness.run_path(&run_id).join("events.jsonl"),
        b"{\"schema_version\":2,\"timestamp_ms\":\"invalid\"}\n",
    )?;
    assert_failure(
        &harness.run(&["measure", "report", "--format", "json"])?,
        "events.jsonl has an invalid record",
    );
    Ok(())
}
#[test]
fn reports_unavailable_rates_as_null_and_never_exposes_private_context()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new_v2()?;
    let private = "fixture-private-context";
    harness.capture("codex", "session_id", "pending", private)?;
    let output = harness.run(&["measure", "report", "--format", "json"])?;
    assert_success(&output);
    let report: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        *(*(report)
            .get("totals")
            .ok_or("missing fixture index totals")?)
        .get("judgeable_rate")
        .ok_or("missing fixture index judgeable_rate")?,
        0.0
    );
    assert!(
        (*(*(report)
            .get("totals")
            .ok_or("missing fixture index totals")?)
        .get("declared_success_rate")
        .ok_or("missing fixture index declared_success_rate")?)
        .is_null()
    );
    assert_eq!(
        (*(report)
            .get("agents")
            .ok_or("missing fixture index agents")?)
        .as_array()
        .ok_or("expected JSON array")?
        .len(),
        1
    );
    assert_eq!(
        *(*(*(report)
            .get("agents")
            .ok_or("missing fixture index agents")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("agent")
        .ok_or("missing fixture index agent")?,
        "codex"
    );
    assert!(!String::from_utf8(output.stdout)?.contains(private));
    Ok(())
}
fn record_outcome(
    harness: &Harness,
    run_id: &str,
    status: &str,
    fields: &[&str],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut arguments = vec!["measure", "outcome", run_id, "--status", status];
    arguments.extend_from_slice(fields);
    assert_success(&harness.run(&arguments)?);
    Ok(())
}
fn set_times(
    harness: &Harness,
    run_id: &str,
    started_at_ms: u64,
    events: &[u64],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let run = harness.run_path(run_id);
    let mut metadata = read_json(run.join("run.json"))?;
    {
        metadata
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("started_at_ms").to_owned(), json!(started_at_ms));
    };
    fs::write(run.join("run.json"), serde_json::to_vec_pretty(&metadata)?)?;
    let mut records = read_jsonl(run.join("events.jsonl"))?;
    for (record, timestamp) in records.iter_mut().zip(events) {
        {
            record
                .as_object_mut()
                .ok_or("expected JSON object for fixture update")?
                .insert(("timestamp_ms").to_owned(), json!(timestamp));
        };
    }
    let bytes = records
        .iter()
        .map(
            |record| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(serde_json::to_string(record)?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(run.join("events.jsonl"), bytes)?;
    Ok(())
}
