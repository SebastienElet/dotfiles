use super::super::model::{PromptRecord, RunRecord};
use super::super::{MeasureError, outcome::latest as latest_outcome};
use super::io::read_optional_json;
use super::records::{
    ResultState, read_events_for_list_with, read_events_with, read_first_prompt, result_state,
};
use super::{ListArgs, ListFormat, ResultRecord, open_run, open_store, validate_result_record};
use crate::measure::hook::now_ms;
use serde::Serialize;

#[derive(Serialize)]
struct ListedRun {
    run_id: String,
    agent: String,
    repository: Option<String>,
    first_prompt_excerpt: Option<String>,
    last_event: Option<String>,
    #[serde(skip)]
    first_event_at_ms: Option<u64>,
    #[serde(skip)]
    last_event_at_ms: Option<u64>,
    #[serde(skip)]
    event_timestamps_consistent: bool,
    has_result: bool,
    has_outcome: bool,
    result_state: ResultState,
    started_at_ms: u64,
}

#[derive(Serialize)]
struct SilenceReport {
    reported_at_ms: u64,
    runs: Vec<SilentRun>,
}
#[derive(Serialize)]
struct SilentRun {
    run_id: String,
    agent: String,
    started_at_ms: u64,
    last_event: Option<String>,
    last_event_at_ms: Option<u64>,
    start_to_last_event_ms: Option<u64>,
    silence_ms: Option<u64>,
}

pub fn render(args: ListArgs) -> Result<String, MeasureError> {
    let store = open_store()?;
    let runs_path = store.runs_path();
    let mut runs = if runs_path.exists()? {
        runs_path.open_directory()?;
        collect(&store, args.without_result)?
    } else {
        Vec::new()
    };
    if let Some(agent) = args.agent {
        runs.retain(|run| run.agent == agent.as_str());
    }
    if args.without_result {
        return render_silence(runs, args.format, now_ms());
    }
    runs.sort_by(|left, right| {
        left.started_at_ms
            .cmp(&right.started_at_ms)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    match args.format {
        ListFormat::Json => serde_json::to_string_pretty(&runs).map_err(Into::into),
        ListFormat::Human => Ok(render_human(&runs)),
    }
}

fn collect(
    store: &super::super::store::Store,
    allow_empty_events: bool,
) -> Result<Vec<ListedRun>, MeasureError> {
    let mut runs = Vec::new();
    for name in store.runs_path().read_dir_names()? {
        let run_id = name
            .into_string()
            .map_err(|_| MeasureError::new("managed run id is not UTF-8"))?;
        let run_dir = open_run(store, &run_id)?;
        let lifecycle = store.open_run_lock(&run_id)?;
        lifecycle.lock()?;
        let run: RunRecord = super::super::store::validation::read_run(&run_dir.join("run.json"))?;
        let first_prompt = read_first_prompt(&run_dir.join("prompts.jsonl"), &run)?;
        let read_result = || read_optional_json(&run_dir.join("result.json"), "result.json");
        let (events, result): (_, Option<ResultRecord>) = if allow_empty_events {
            read_events_for_list_with(&run_dir.join("events.jsonl"), &run_id, read_result)?
        } else {
            read_events_with(&run_dir.join("events.jsonl"), &run_id, read_result)?
        };
        if let Some(result) = &result {
            validate_result_record(result, &run_id)?;
        }
        let result_state = result_state(&events, result.as_ref())?;
        let outcome = latest_outcome(&run_dir.join("outcomes.jsonl"), &run_id)?;
        let has_outcome = outcome.is_some();
        let result_state = match (run.schema_version(), result_state, has_outcome) {
            (1, state, false) => state,
            (2, ResultState::Pending, true) => ResultState::OutcomeRecorded,
            (2, ResultState::Pending, false) => ResultState::Pending,
            _ => {
                return Err(MeasureError::new(
                    "managed run mixes incompatible result contracts",
                ));
            }
        };
        runs.push(ListedRun {
            run_id,
            agent: run.agent().to_owned(),
            repository: run.repository().map(str::to_owned),
            first_prompt_excerpt: first_prompt.as_ref().map(first_prompt_excerpt),
            last_event: events.last_event().map(str::to_owned),
            first_event_at_ms: events.first_event_at_ms(),
            last_event_at_ms: events.last_event_at_ms(),
            event_timestamps_consistent: events.timestamps_consistent(),
            has_result: result.is_some(),
            has_outcome,
            result_state,
            started_at_ms: run.started_at_ms(),
        });
    }
    Ok(runs)
}

fn render_silence(
    runs: Vec<ListedRun>,
    format: ListFormat,
    reported_at_ms: u64,
) -> Result<String, MeasureError> {
    let mut runs = runs
        .into_iter()
        .filter(|run| run.result_state == ResultState::Pending)
        .map(|run| SilentRun::new(run, reported_at_ms))
        .collect::<Vec<_>>();
    runs.sort_by(|left, right| {
        right
            .silence_ms
            .cmp(&left.silence_ms)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    let report = SilenceReport {
        reported_at_ms,
        runs,
    };
    match format {
        ListFormat::Json => serde_json::to_string_pretty(&report).map_err(Into::into),
        ListFormat::Human => Ok(render_human_silence(&report)),
    }
}

impl SilentRun {
    fn new(run: ListedRun, reported_at_ms: u64) -> Self {
        let durations = run
            .last_event_at_ms
            .filter(|last| {
                run.event_timestamps_consistent
                    && run
                        .first_event_at_ms
                        .is_some_and(|first| run.started_at_ms <= first)
                    && *last <= reported_at_ms
            })
            .map(|last| (last - run.started_at_ms, reported_at_ms - last));
        Self {
            run_id: run.run_id,
            agent: run.agent,
            started_at_ms: run.started_at_ms,
            last_event: run.last_event,
            last_event_at_ms: run.last_event_at_ms,
            start_to_last_event_ms: durations.map(|value| value.0),
            silence_ms: durations.map(|value| value.1),
        }
    }
}

fn render_human_silence(report: &SilenceReport) -> String {
    std::iter::once(format!("reported_at_ms={}", report.reported_at_ms))
        .chain(report.runs.iter().map(|run| {
            format!(
                "{} agent={} started_at_ms={} last_event={} last_event_at_ms={} start_to_last_event_ms={} silence_ms={}",
                run.run_id,
                run.agent,
                run.started_at_ms,
                optional_text(run.last_event.as_deref()),
                optional_u64(run.last_event_at_ms),
                optional_u64(run.start_to_last_event_ms),
                optional_u64(run.silence_ms)
            )
        }))
        .map(|line| escape_terminal_controls(&line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn optional_text(value: Option<&str>) -> &str {
    value.unwrap_or("unavailable")
}

fn optional_u64(value: Option<u64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn first_prompt_excerpt(record: &PromptRecord) -> String {
    excerpt(&record.prompt)
}

fn excerpt(value: &str) -> String {
    let normalized = value.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut characters = normalized.chars();
    let excerpt: String = characters.by_ref().take(120).collect();
    if characters.next().is_some() {
        format!("{excerpt}…")
    } else {
        excerpt
    }
}

fn render_human(runs: &[ListedRun]) -> String {
    runs.iter()
        .map(|run| {
            escape_terminal_controls(&format!(
                "{} {} repository={} result={} last={} prompt={}",
                run.run_id,
                run.agent,
                run.repository.as_deref().unwrap_or("-"),
                run.result_state.as_str(),
                run.last_event.as_deref().unwrap_or("-"),
                run.first_prompt_excerpt.as_deref().unwrap_or("-")
            ))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn escape_terminal_controls(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                format!("\\u{{{:04x}}}", character as u32)
            } else {
                character.to_string()
            }
        })
        .collect()
}
