use super::super::MeasureError;
use super::super::model::{PromptRecord, RunRecord};
use super::io::read_optional_json;
use super::records::{StoredEvent, read_events, read_prompts, validate_result_coherence};
use super::{ListArgs, ListFormat, ResultRecord, open_run, open_store, validate_result_record};
use serde::Serialize;
use std::fs;

#[derive(Serialize)]
struct ListedRun {
    run_id: String,
    agent: String,
    repository: Option<String>,
    first_prompt_excerpt: Option<String>,
    last_event: Option<String>,
    has_result: bool,
    started_at_ms: u64,
}

pub fn render(args: ListArgs) -> Result<String, MeasureError> {
    let store = open_store()?;
    let runs_path = store.runs_path();
    let mut runs = match fs::symlink_metadata(&runs_path) {
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => collect(&store)?,
        Ok(_) => {
            return Err(MeasureError::new(
                "managed runs path is not a real directory",
            ));
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
        Err(error) => return Err(error.into()),
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
    for entry in fs::read_dir(store.runs_path())? {
        let entry = entry?;
        let run_id = entry
            .file_name()
            .into_string()
            .map_err(|_| MeasureError::new("managed run id is not UTF-8"))?;
        let run_dir = open_run(store, &run_id)?;
        let run: RunRecord = super::super::store::validation::read_run(&run_dir.join("run.json"))?;
        let prompts = read_prompts(&run_dir.join("prompts.jsonl"), &run)?;
        let events = read_events(&run_dir.join("events.jsonl"), &run_id)?;
        let result: Option<ResultRecord> =
            read_optional_json(&run_dir.join("result.json"), "result.json")?;
        if let Some(result) = &result {
            validate_result_record(result, &run_id)?;
        }
        validate_result_coherence(&events, result.as_ref())?;
        runs.push(ListedRun {
            run_id,
            agent: run.agent,
            repository: run.repository,
            first_prompt_excerpt: first_prompt(&prompts),
            last_event: last_event(&events),
            has_result: result.is_some(),
            started_at_ms: run.started_at_ms,
        });
    }
    Ok(runs)
}

fn first_prompt(records: &[PromptRecord]) -> Option<String> {
    records.first().map(|record| excerpt(&record.prompt))
}

fn last_event(records: &[StoredEvent]) -> Option<String> {
    records.last().map(|record| record.event.clone())
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
            format!(
                "{} {} repository={} result={} last={} prompt={}",
                run.run_id,
                run.agent,
                run.repository.as_deref().unwrap_or("-"),
                if run.has_result {
                    "recorded"
                } else {
                    "pending"
                },
                run.last_event.as_deref().unwrap_or("-"),
                run.first_prompt_excerpt.as_deref().unwrap_or("-")
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}
