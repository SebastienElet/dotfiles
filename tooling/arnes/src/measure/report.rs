use super::MeasureError;
use super::hook::now_ms;
use super::outcome::{OutcomeRecord, OutcomeStatus, latest as latest_outcome};
use super::result::{EventHistory, ListFormat, open_run, open_store, read_events_for_list_with};
use super::store::{StorageUsage, validation};
use clap::Args;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Args)]
pub struct ReportArgs {
    #[arg(long, value_enum, default_value_t)]
    pub format: ListFormat,
}

#[derive(Default, Serialize)]
struct Counts {
    runs: u64,
    judgeable_runs: u64,
    judgeable_rate: Option<f64>,
    declared_successful_runs: u64,
    declared_success_rate: Option<f64>,
    event_count: u64,
    tool_call_count: u64,
    latency_runs: u64,
    mean_observed_latency_ms: Option<f64>,
    #[serde(skip)]
    latency_total_ms: u64,
}

#[derive(Serialize)]
struct AgentMetrics {
    agent: String,
    metrics: Counts,
}

#[derive(Serialize)]
struct Report {
    reported_at_ms: u64,
    totals: Counts,
    agents: Vec<AgentMetrics>,
    storage: StorageUsage,
}

pub fn render(args: ReportArgs) -> Result<String, MeasureError> {
    let reported_at_ms = now_ms();
    let store = open_store()?;
    let mut totals = Counts::default();
    let mut agents = BTreeMap::<String, Counts>::new();
    let runs = store.runs_path();
    if runs.exists()? {
        runs.open_directory()?;
        for name in runs.read_dir_names()? {
            let run_id = name
                .into_string()
                .map_err(|_| MeasureError::new("managed run id is not UTF-8"))?;
            let run = open_run(&store, &run_id)?;
            let lifecycle = store.open_run_lock(&run_id)?;
            lifecycle.lock()?;
            let metadata = validation::read_run(&run.join("run.json"))?;
            let (events, ()) =
                read_events_for_list_with(&run.join("events.jsonl"), &run_id, || Ok(()))?;
            let outcome = latest_outcome(&run.join("outcomes.jsonl"), &run_id)?;
            let latency = observed_latency(metadata.started_at_ms(), &events, reported_at_ms);
            add_run(&mut totals, &events, outcome.as_ref(), latency);
            add_run(
                agents.entry(metadata.agent().to_owned()).or_default(),
                &events,
                outcome.as_ref(),
                latency,
            );
        }
    }
    finalize(&mut totals);
    let report = Report {
        reported_at_ms,
        totals,
        agents: ordered_agents(agents),
        storage: store.usage()?,
    };
    match args.format {
        ListFormat::Json => serde_json::to_string_pretty(&report).map_err(Into::into),
        ListFormat::Human => Ok(render_human(&report)),
    }
}

fn add_run(
    counts: &mut Counts,
    events: &EventHistory,
    outcome: Option<&OutcomeRecord>,
    latency: Option<u64>,
) {
    counts.runs += 1;
    counts.event_count += events.event_count();
    counts.tool_call_count += events.tool_call_count();
    if let Some(outcome) = outcome {
        if matches!(outcome.status(), OutcomeStatus::Pass | OutcomeStatus::Fail) {
            counts.judgeable_runs += 1;
        }
        if outcome.status() == OutcomeStatus::Pass {
            counts.declared_successful_runs += 1;
        }
    }
    if let Some(latency) = latency {
        counts.latency_runs += 1;
        counts.latency_total_ms = counts.latency_total_ms.saturating_add(latency);
    }
}

fn observed_latency(started_at_ms: u64, events: &EventHistory, reported_at_ms: u64) -> Option<u64> {
    events
        .last_event_at_ms()
        .filter(|last| {
            events.timestamps_consistent() && started_at_ms <= *last && *last <= reported_at_ms
        })
        .map(|last| last - started_at_ms)
}

fn finalize(counts: &mut Counts) {
    counts.judgeable_rate = ratio(counts.judgeable_runs, counts.runs);
    counts.declared_success_rate = ratio(counts.declared_successful_runs, counts.judgeable_runs);
    counts.mean_observed_latency_ms = ratio(counts.latency_total_ms, counts.latency_runs);
}

fn ratio(numerator: u64, denominator: u64) -> Option<f64> {
    (denominator > 0).then(|| numerator as f64 / denominator as f64)
}

fn ordered_agents(mut agents: BTreeMap<String, Counts>) -> Vec<AgentMetrics> {
    ["codex", "claude-code", "cursor"]
        .into_iter()
        .filter_map(|agent| {
            let mut metrics = agents.remove(agent)?;
            finalize(&mut metrics);
            Some(AgentMetrics {
                agent: agent.to_owned(),
                metrics,
            })
        })
        .collect()
}

fn render_human(report: &Report) -> String {
    let totals = &report.totals;
    format!(
        "runs={} judgeable={} judgeable_rate={} declared_success={} declared_success_rate={} events={} tool_calls={} latency_runs={} mean_latency_ms={} logical_bytes={} allocated_bytes={}",
        totals.runs,
        totals.judgeable_runs,
        optional_rate(totals.judgeable_rate),
        totals.declared_successful_runs,
        optional_rate(totals.declared_success_rate),
        totals.event_count,
        totals.tool_call_count,
        totals.latency_runs,
        optional_rate(totals.mean_observed_latency_ms),
        report.storage.logical_bytes,
        report.storage.allocated_bytes
    )
}

fn optional_rate(value: Option<f64>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
}
