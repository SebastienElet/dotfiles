use super::{
    evidence::validate_report,
    report::{Harness, Report, Status},
};
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metrics {
    pub pass_rate: f64,
    pub failures: usize,
    pub invalid: usize,
    pub mean_tokens: Option<f64>,
    pub mean_tool_calls: Option<f64>,
    pub mean_duration_ms: Option<f64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Regression {
    pub case_id: String,
    pub run: usize,
}
#[derive(Debug, Serialize)]
pub struct HarnessComparison {
    pub baseline: Harness,
    pub candidate: Harness,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub comparable: bool,
    pub claim: String,
    pub baseline: Metrics,
    pub candidate: Metrics,
    pub pass_rate_delta: f64,
    pub regressions: Vec<Regression>,
    pub harness: HarnessComparison,
    pub limitations: Vec<String>,
}

pub fn metrics(report: &Report) -> Metrics {
    let runs = report
        .cases
        .iter()
        .flat_map(|entry| &entry.runs)
        .collect::<Vec<_>>();
    let mean = |values: Vec<Option<f64>>| {
        values
            .into_iter()
            .collect::<Option<Vec<_>>>()
            .map(|values| values.iter().sum::<f64>() / values.len() as f64)
    };
    Metrics {
        pass_rate: runs.iter().filter(|run| run.status == Status::Pass).count() as f64
            / runs.len() as f64,
        failures: runs.iter().filter(|run| run.status == Status::Fail).count(),
        invalid: runs
            .iter()
            .filter(|run| run.status == Status::Invalid)
            .count(),
        mean_tokens: mean(
            runs.iter()
                .map(|r| r.tokens.as_ref().map(|t| t.input as f64 + t.output as f64))
                .collect(),
        ),
        mean_tool_calls: mean(
            runs.iter()
                .map(|r| r.tool_calls.map(|n| n as f64))
                .collect(),
        ),
        mean_duration_ms: mean(runs.iter().map(|r| r.duration_ms).collect()),
    }
}

fn same_controls(a: &Report, b: &Report) -> bool {
    a.agent == b.agent
        && a.agent_version == b.agent_version
        && a.model == b.model
        && a.runner_revision == b.runner_revision
        && a.environment == b.environment
        && a.controls == b.controls
        && a.run_count == b.run_count
        && a.cases.len() == b.cases.len()
        && a.cases.iter().zip(&b.cases).all(|(a, b)| {
            a.definition == b.definition
                && a.prompt_fingerprint == b.prompt_fingerprint
                && a.fixture_revision == b.fixture_revision
        })
}

pub fn compare(baseline: &Report, candidate: &Report) -> Result<Comparison, String> {
    validate_report(baseline)?;
    validate_report(candidate)?;
    if !same_controls(baseline, candidate) {
        return Err("Reports are not comparable: cases, prompts, fixtures, runner, model, environment, permissions or budgets differ".into());
    }
    let before = metrics(baseline);
    let after = metrics(candidate);
    let regressions = baseline
        .cases
        .iter()
        .zip(&candidate.cases)
        .flat_map(|(a, b)| {
            a.runs
                .iter()
                .zip(&b.runs)
                .enumerate()
                .filter(|(_, (a_run, b_run))| {
                    a_run.status == Status::Pass && b_run.status != Status::Pass
                })
                .map(|(i, _)| Regression {
                    case_id: a.definition.id.clone(),
                    run: i + 1,
                })
        })
        .collect();
    Ok(Comparison {
        comparable: true,
        claim: "descriptive-only".into(),
        pass_rate_delta: after.pass_rate - before.pass_rate,
        baseline: before, candidate: after, regressions,
        harness: HarnessComparison { baseline: baseline.harness.clone(), candidate: candidate.harness.clone() },
        limitations: vec![
            "Matching recorded controls does not establish causality or statistical significance.".into(),
            "Replicate positions are paired descriptively; Codex does not expose paired random seeds.".into(),
            "INVALID runs remain in the denominator; fixture-smoke is not live behavioral evidence.".into(),
        ],
    })
}
