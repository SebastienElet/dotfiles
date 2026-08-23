use super::super::MeasureError;
use super::super::model::{PromptRecord, RunRecord};
use super::io::read_optional_json;
use super::records::{ResultState, read_events_with, read_first_prompt, result_state};
use super::{ListArgs, ListFormat, ResultRecord, open_run, open_store, validate_result_record};
use serde::Serialize;

#[derive(Serialize)]
struct ListedRun {
    run_id: String,
    agent: String,
    repository: Option<String>,
    first_prompt_excerpt: Option<String>,
    last_event: Option<String>,
    has_result: bool,
    result_state: ResultState,
    started_at_ms: u64,
}

pub fn render(args: ListArgs) -> Result<String, MeasureError> {
    let store = open_store()?;
    let runs_path = store.runs_path();
    let mut runs = if runs_path.exists()? {
        runs_path.open_directory()?;
        collect(&store)?
    } else {
        Vec::new()
    };
    if let Some(agent) = args.agent {
        runs.retain(|run| run.agent == agent.as_str());
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

fn collect(store: &super::super::store::Store) -> Result<Vec<ListedRun>, MeasureError> {
    let mut runs = Vec::new();
    for name in store.runs_path().read_dir_names()? {
        let run_id = name
            .into_string()
            .map_err(|_| MeasureError::new("managed run id is not UTF-8"))?;
        let run_dir = open_run(store, &run_id)?;
        let run: RunRecord = super::super::store::validation::read_run(&run_dir.join("run.json"))?;
        let first_prompt = read_first_prompt(&run_dir.join("prompts.jsonl"), &run)?;
        let (events, result): (_, Option<ResultRecord>) =
            read_events_with(&run_dir.join("events.jsonl"), &run_id, || {
                read_optional_json(&run_dir.join("result.json"), "result.json")
            })?;
        if let Some(result) = &result {
            validate_result_record(result, &run_id)?;
        }
        let result_state = result_state(&events, result.as_ref())?;
        runs.push(ListedRun {
            run_id,
            agent: run.agent,
            repository: run.repository,
            first_prompt_excerpt: first_prompt.as_ref().map(first_prompt_excerpt),
            last_event: events.last_event().map(str::to_owned),
            has_result: result.is_some(),
            result_state,
            started_at_ms: run.started_at_ms,
        });
    }
    Ok(runs)
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
