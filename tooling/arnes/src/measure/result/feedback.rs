use super::super::MeasureError;
use super::super::hook::now_ms;
use super::super::store::{append_jsonl, open_private_append};
use super::io::read_jsonl_typed;
use super::{Adjudication, FeedbackArgs, Severity, open_run, open_store};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct FeedbackRecord {
    schema_version: u8,
    feedback_id: String,
    run_id: String,
    recorded_at_ms: u64,
    source_type: super::FeedbackSource,
    source_id: String,
    scope: String,
    observed: String,
    expected: String,
    evidence: Vec<String>,
    invariants: Vec<String>,
    severity: Severity,
    adjudication: Adjudication,
    resolution: super::Resolution,
    failure_category: super::FailureCategory,
    analysis_blocking: bool,
}

pub fn record(args: FeedbackArgs) -> Result<(), MeasureError> {
    validate(&args)?;
    let store = open_store()?;
    let run_dir = open_run(&store, &args.run_id)?;
    let timestamp = now_ms();
    let analysis_blocking =
        args.severity == Severity::Blocking && args.adjudication == Adjudication::Confirmed;
    let feedback = FeedbackRecord {
        schema_version: 1,
        feedback_id: feedback_id(&args, timestamp),
        run_id: args.run_id,
        recorded_at_ms: timestamp,
        source_type: args.source_type,
        source_id: args.source_id,
        scope: args.scope,
        observed: args.observed,
        expected: args.expected,
        evidence: args.evidence,
        invariants: args.invariant,
        severity: args.severity,
        adjudication: args.adjudication,
        resolution: args.resolution,
        failure_category: args.failure_category,
        analysis_blocking,
    };
    let path = run_dir.join("feedback.jsonl");
    let lock = open_private_append(&run_dir.join("feedback.lock"))?;
    lock.lock()?;
    validate_existing(&path, &feedback.run_id)?;
    append_jsonl(&path, &feedback)
}

fn validate(args: &FeedbackArgs) -> Result<(), MeasureError> {
    for (label, value) in [
        ("source id", args.source_id.as_str()),
        ("scope", args.scope.as_str()),
        ("observed", args.observed.as_str()),
        ("expected", args.expected.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(MeasureError::new(format!("{label} cannot be empty")));
        }
    }
    if args
        .evidence
        .iter()
        .chain(&args.invariant)
        .any(|value| value.trim().is_empty())
    {
        return Err(MeasureError::new(
            "evidence and invariant references cannot be empty",
        ));
    }
    Ok(())
}

fn validate_existing(path: &std::path::Path, run_id: &str) -> Result<(), MeasureError> {
    for feedback in read_jsonl_typed::<FeedbackRecord>(path, "feedback.jsonl")? {
        if feedback.schema_version != 1
            || feedback.run_id != run_id
            || feedback.feedback_id.len() != 64
            || feedback.recorded_at_ms == 0
            || feedback.source_id.trim().is_empty()
            || feedback.scope.trim().is_empty()
            || feedback.observed.trim().is_empty()
            || feedback.expected.trim().is_empty()
            || feedback.analysis_blocking
                != (feedback.severity == Severity::Blocking
                    && feedback.adjudication == Adjudication::Confirmed)
            || has_empty_reference(&feedback)
        {
            return Err(MeasureError::new(
                "managed feedback.jsonl has an invalid record",
            ));
        }
    }
    Ok(())
}

fn has_empty_reference(feedback: &FeedbackRecord) -> bool {
    feedback
        .evidence
        .iter()
        .chain(&feedback.invariants)
        .any(|value| value.trim().is_empty())
}

fn feedback_id(args: &FeedbackArgs, timestamp: u64) -> String {
    let mut hasher = Sha256::new();
    for value in [
        args.run_id.as_bytes(),
        args.source_id.as_bytes(),
        args.observed.as_bytes(),
        timestamp.to_string().as_bytes(),
        std::process::id().to_string().as_bytes(),
    ] {
        hasher.update((value.len() as u64).to_be_bytes());
        hasher.update(value);
    }
    format!("{:x}", hasher.finalize())
}
