use super::evidence::validate_report;
use super::fixture::Fixture;
use super::identity::{date, git_revision, runner_revision};
use super::live::{Codex, Execution, LiveOptions};
use super::observe::observe;
use super::report::{Controls, Environment, Harness, Report, ReportCase};
use super::sources::{LoadedCase, fingerprint, load_cases, read_source, section};
use std::collections::HashSet;
use std::path::Path;

pub struct SeriesOptions {
    pub model: String,
    pub only: Vec<String>,
    pub runs: u64,
    pub timeout_seconds: u64,
    pub reasoning_effort: String,
    pub variant: Option<String>,
}

pub fn run_live(repository: &Path, options: &SeriesOptions) -> Result<Report, String> {
    let selected = select_cases(repository, options)?;
    let codex = Codex::discover()?;
    let live = LiveOptions {
        model: options.model.clone(),
        reasoning_effort: options.reasoning_effort.clone(),
        timeout_seconds: options.timeout_seconds,
    };
    run_series(
        repository,
        options,
        selected,
        "codex",
        &codex.version,
        |fixture, entry| codex.execute(fixture, &entry.prompt, &live),
    )
}

pub fn run_smoke(repository: &Path) -> Result<Report, String> {
    let selected = load_cases(repository)?;
    let options = SeriesOptions {
        model: "none".to_owned(),
        only: selected
            .iter()
            .map(|entry| entry.definition.id.clone())
            .collect(),
        runs: 1,
        timeout_seconds: 120,
        reasoning_effort: "low".to_owned(),
        variant: None,
    };
    run_series(
        repository,
        &options,
        selected,
        "fixture-smoke",
        "1",
        super::smoke::execute,
    )
}

fn select_cases(repository: &Path, options: &SeriesOptions) -> Result<Vec<LoadedCase>, String> {
    let available = load_cases(repository)?;
    let unique: HashSet<_> = options.only.iter().collect();
    if unique.is_empty()
        || unique.len() != options.only.len()
        || unique
            .iter()
            .any(|id| !available.iter().any(|entry| &entry.definition.id == *id))
    {
        return Err("Explicit, unique known case selection required".to_owned());
    }
    if !(1..=10).contains(&options.runs) || !(1..=600).contains(&options.timeout_seconds) {
        return Err("Runs must be 1–10 and timeout seconds 1–600".to_owned());
    }
    Ok(available
        .into_iter()
        .filter(|entry| unique.contains(&entry.definition.id))
        .collect())
}

fn run_series(
    repository: &Path,
    options: &SeriesOptions,
    selected: Vec<LoadedCase>,
    agent: &str,
    version: &str,
    mut execute: impl FnMut(&Fixture, &LoadedCase) -> Result<Execution, String>,
) -> Result<Report, String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let revision = runner_revision(&executable)?;
    let git = git_revision(repository)?;
    let mut identity = None;
    let mut cases = Vec::new();
    for entry in selected {
        let mut runs = Vec::new();
        for _ in 0..options.runs {
            let (fixture, harness) = prepare(repository, options, &entry, &executable, &git)?;
            if identity
                .as_ref()
                .is_some_and(|previous| previous != &harness)
            {
                return Err("Harness changed during evaluation".to_owned());
            }
            identity = Some(harness);
            runs.push(observe(&fixture, &entry, execute(&fixture, &entry)?));
        }
        let fixture_revision = fingerprint(format!(
            "{}\0{revision}",
            serde_json::to_string(&entry.fixture).map_err(|error| error.to_string())?
        ));
        cases.push(ReportCase {
            definition: entry.definition,
            prompt: entry.prompt,
            prompt_fingerprint: entry.prompt_fingerprint,
            sources: entry.sources,
            fixture_revision,
            runs,
        });
    }
    if runner_revision(&executable)? != revision {
        return Err("Evaluation executable changed during execution".to_owned());
    }
    let report = report(
        options,
        agent,
        version,
        identity.ok_or("No evaluation runs")?,
        revision,
        cases,
    )?;
    validate_report(&report)?;
    Ok(report)
}

fn prepare(
    repository: &Path,
    options: &SeriesOptions,
    entry: &LoadedCase,
    executable: &Path,
    git: &str,
) -> Result<(Fixture, Harness), String> {
    let instructions = match &options.variant {
        Some(path) => read_source(repository, path)?,
        None => section(
            &read_source(repository, "harness/AGENTS.md")?,
            "Context Management",
        )?
        .to_owned(),
    };
    let skill = read_source(repository, "harness/skills/code-search/SKILL.md")?;
    let harness = Harness {
        git_revision: git.to_owned(),
        instruction_fingerprint: fingerprint(&instructions),
        skill_fingerprint: fingerprint(&skill),
        variant: options
            .variant
            .clone()
            .unwrap_or_else(|| "Context Management".to_owned()),
    };
    Ok((
        Fixture::prepare(&entry.fixture.files, &instructions, &skill, executable)?,
        harness,
    ))
}

fn report(
    options: &SeriesOptions,
    agent: &str,
    version: &str,
    harness: Harness,
    revision: String,
    cases: Vec<ReportCase>,
) -> Result<Report, String> {
    Ok(Report {
        schema_version: 1, agent: agent.to_owned(), agent_version: version.to_owned(),
        model: options.model.clone(), date: date()?, harness, runner_revision: revision,
        environment: Environment::Rust {
            platform: std::env::consts::OS.to_owned(), architecture: std::env::consts::ARCH.to_owned(),
            runtime: format!("arnes-rust-{}", env!("CARGO_PKG_VERSION")),
        },
        controls: Controls {
            sandbox: "workspace-write".to_owned(), network: false,
            tools: "shell-with-synthetic-cat-rg-fd-colgrep-v1".to_owned(),
            timeout_seconds: options.timeout_seconds, reasoning_effort: options.reasoning_effort.clone(), token_budget: (),
        },
        run_count: options.runs as usize, cases,
        limitations: [
            "Only Context Management and code-search are installed; this is not the full deployed harness.",
            "PATH shims observe supported commands, not internal skill loading or uninstrumented reads; bypasses can cause false negatives.",
            "Synthetic shims are not ColGrep quality or security tests; observations are not tamper-proof.",
            "Git revision plus content fingerprints identify the tested bytes, including uncommitted changes.",
            "Model is the requested ID; aliases may resolve differently. No statistical or causal uplift claim.",
            "No token ceiling is available; the wall-clock timeout bounds each run. Raw transcripts are discarded.",
        ].map(str::to_owned).to_vec(),
    })
}
