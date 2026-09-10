use super::measure_support::*;
use serde_json::Value;
#[test]
fn finish_rejects_invalid_oracle_combinations()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    for (arguments, expected) in [
        (
            vec!["--merge-ready", "pass", "--human-minutes", "-1"],
            "finite non-negative",
        ),
        (
            vec!["--merge-ready", "pass", "--human-minutes", "NaN"],
            "finite non-negative",
        ),
        (
            vec!["--merge-ready", "fail", "--human-minutes", "1"],
            "failure reason is required",
        ),
        (
            vec![
                "--merge-ready",
                "pass",
                "--human-minutes",
                "1",
                "--failure-reason",
                "not actually ready",
            ],
            "failure reason is forbidden",
        ),
        (
            vec!["--merge-ready", "unjudgeable", "--human-minutes", "1"],
            "evidence is required",
        ),
    ] {
        let mut command = vec!["measure", "finish", &run_id];
        command.extend(arguments);
        assert_failure(&harness.run(&command)?, expected);
    }
    assert!(!harness.run_path(&run_id).join("result.json").exists());
    Ok(())
}
#[test]
fn repeated_finish_increments_revision_and_keeps_each_adjudication_in_events()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let first = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "fail",
        "--human-minutes",
        "7",
        "--human-edited-diff",
        "--failure-reason",
        "tests failed",
        "--evidence",
        "cargo test",
        "--regression",
        "--invariant",
        "tests-green",
    ])?;
    assert_success(&first);
    let second = harness.run(&[
        "measure",
        "finish",
        &run_id,
        "--merge-ready",
        "pass",
        "--human-minutes",
        "9",
        "--evidence",
        "fixed and rerun",
        "--invariant",
        "tests-green",
    ])?;
    assert_success(&second);
    let result = read_json(harness.run_path(&run_id).join("result.json"))?;
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
    assert_eq!(
        *(result)
            .get("human_minutes")
            .ok_or("missing fixture index human_minutes")?,
        9.0
    );
    let events = read_jsonl(harness.run_path(&run_id).join("events.jsonl"))?;
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
    for (index, revision, verdict) in [(0, 1, "fail"), (1, 2, "pass")] {
        let decision = decisions.get(index).ok_or("missing adjudication event")?;
        assert_eq!(
            decision.pointer("/result/revision"),
            Some(&serde_json::json!(revision))
        );
        assert_eq!(
            decision.pointer("/result/merge_ready"),
            Some(&serde_json::json!(verdict))
        );
    }

    Ok(())
}
#[test]
fn parallel_finish_calls_keep_unique_contiguous_revisions()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let run_id = harness.capture("codex", "session_id", "session", "prompt")?;
    let mut children = Vec::new();
    for minutes in 1..=8 {
        children.push(
            harness
                .command()
                .args([
                    "measure",
                    "finish",
                    &run_id,
                    "--merge-ready",
                    "pass",
                    "--human-minutes",
                    &minutes.to_string(),
                ])
                .spawn()?,
        );
    }
    for child in children {
        assert_success(&child.wait_with_output()?);
    }
    let result = read_json(harness.run_path(&run_id).join("result.json"))?;
    assert_eq!(
        *(result)
            .get("revision")
            .ok_or("missing fixture index revision")?,
        8
    );
    let events = read_jsonl(harness.run_path(&run_id).join("events.jsonl"))?;
    let mut revisions: Vec<u64> = events
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
        .map(
            |event| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok((*(*(event)
                    .get("result")
                    .ok_or("missing fixture index result")?)
                .get("revision")
                .ok_or("missing fixture index revision")?)
                .as_u64()
                .ok_or("expected JSON unsigned integer")?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    revisions.sort_unstable();
    assert_eq!(revisions, (1..=8).collect::<Vec<_>>());
    Ok(())
}
