use super::measure_support::*;

#[test]
fn finish_refuses_v2_runs_in_favor_of_explicit_outcomes() {
    let harness = Harness::new_v2();
    let run_id = harness.capture("codex", "session_id", "session", "fixture prompt");

    assert_failure(
        &harness.run(&[
            "measure",
            "finish",
            &run_id,
            "--merge-ready",
            "pass",
            "--human-minutes",
            "1",
        ]),
        "measure finish supports only v1 runs; use measure outcome",
    );
    assert!(!harness.run_path(&run_id).join("result.json").exists());
}

#[test]
fn feedback_refuses_v2_runs_without_a_reported_evaluation_use() {
    let harness = Harness::new_v2();
    let run_id = harness.capture("codex", "session_id", "session", "fixture prompt");

    assert_failure(
        &harness.run(&[
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
            "detail",
            "--expected",
            "expected",
            "--severity",
            "note",
            "--adjudication",
            "pending",
            "--resolution",
            "open",
            "--failure-category",
            "other",
        ]),
        "measure feedback supports only v1 runs",
    );
    assert!(!harness.run_path(&run_id).join("feedback.jsonl").exists());
}
