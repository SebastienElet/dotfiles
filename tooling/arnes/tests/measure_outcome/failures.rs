use super::measure_support::*;
#[test]
fn validates_status_specific_fields_before_opening_the_store()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new_v2()?;
    let run_id = "a".repeat(64);
    for (arguments, expected) in [
        (
            vec!["--status", "pass"],
            "oracle is required for pass and fail outcomes",
        ),
        (
            vec!["--status", "unjudgeable"],
            "reason is required for unjudgeable outcomes",
        ),
        (
            vec!["--status", "unjudgeable", "--oracle", "cargo-test"],
            "oracle is forbidden for unjudgeable outcomes",
        ),
        (
            vec![
                "--status",
                "pass",
                "--oracle",
                "cargo-test",
                "--reason",
                "missing-oracle",
            ],
            "reason is forbidden for pass and fail outcomes",
        ),
        (
            vec!["--status", "pass", "--oracle", "invalid oracle"],
            "oracle must be a lowercase ASCII identifier",
        ),
    ] {
        let mut command = vec!["measure", "outcome", &run_id];
        command.extend(arguments);
        assert_failure(&harness.run(&command)?, expected);
    }
    assert!(!harness.state_root().exists());
    Ok(())
}
#[test]
fn rejects_unknown_runs_and_truncated_outcome_history_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new_v2()?;
    let unknown = "a".repeat(64);
    assert_failure(
        &harness.run(&[
            "measure",
            "outcome",
            &unknown,
            "--status",
            "pass",
            "--oracle",
            "cargo-test",
        ])?,
        "unknown run",
    );
    let run_id = harness.capture("codex", "session_id", "session", "fixture prompt")?;
    let path = harness.run_path(&run_id).join("outcomes.jsonl");
    std::fs::write(&path, b"{\"partial\"")?;
    let before = std::fs::read(&path)?;
    assert_failure(
        &harness.run(&[
            "measure",
            "outcome",
            &run_id,
            "--status",
            "fail",
            "--oracle",
            "cargo-test",
        ])?,
        "outcomes.jsonl is truncated or oversized",
    );
    assert_eq!(std::fs::read(path)?, before);
    Ok(())
}
#[test]
fn rejects_legacy_runs_that_use_the_finish_contract()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "legacy", "fixture prompt")?;
    assert_failure(
        &harness.run(&[
            "measure",
            "outcome",
            &run_id,
            "--status",
            "pass",
            "--oracle",
            "cargo-test",
        ])?,
        "measure outcome supports only v2 runs; use measure finish",
    );
    Ok(())
}
#[test]
fn rejects_an_outcome_history_with_decreasing_timestamps()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new_v2()?;
    let run_id = harness.capture("codex", "session_id", "session", "fixture prompt")?;
    assert_success(&harness.run(&[
        "measure",
        "outcome",
        &run_id,
        "--status",
        "pass",
        "--oracle",
        "cargo-test",
    ])?);
    let path = harness.run_path(&run_id).join("outcomes.jsonl");
    let mut records = read_jsonl(&path)?;
    let mut second = (*(records).first().ok_or("missing fixture index 0")?).clone();
    {
        second
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("revision").to_owned(), serde_json::json!(2));
    };
    {
        second
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("recorded_at_ms").to_owned(), serde_json::json!(1));
    };
    records.push(second);
    let content = records
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
    std::fs::write(path, content)?;
    assert_failure(
        &harness.run(&[
            "measure",
            "outcome",
            &run_id,
            "--status",
            "pass",
            "--oracle",
            "cargo-test",
        ])?,
        "outcome timestamps must be monotonic",
    );
    Ok(())
}
#[test]
fn rejects_a_replacement_when_the_clock_precedes_the_latest_outcome()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new_v2()?;
    let run_id = harness.capture("codex", "session_id", "session", "fixture prompt")?;
    assert_success(&harness.run(&[
        "measure",
        "outcome",
        &run_id,
        "--status",
        "pass",
        "--oracle",
        "cargo-test",
    ])?);
    let path = harness.run_path(&run_id).join("outcomes.jsonl");
    let mut records = read_jsonl(&path)?;
    {
        (records)
            .get_mut(0)
            .ok_or("missing fixture index 0")?
            .as_object_mut()
            .ok_or("expected JSON object for fixture update")?
            .insert(("recorded_at_ms").to_owned(), serde_json::json!(u64::MAX));
    };
    std::fs::write(
        &path,
        serde_json::to_string((records).first().ok_or("missing fixture index 0")?)? + "\n",
    )?;
    let before = std::fs::read(&path)?;
    assert_failure(
        &harness.run(&[
            "measure",
            "outcome",
            &run_id,
            "--status",
            "fail",
            "--oracle",
            "cargo-test",
            "--replace",
        ])?,
        "outcome timestamps must be monotonic",
    );
    assert_eq!(std::fs::read(path)?, before);
    Ok(())
}
