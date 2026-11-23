mod codex;
mod handoff;
mod model;
mod process;
mod store;
mod supervisor;

use model::{Candidate, Mandate, Phase, Resolution, State};
use std::{io::Write, path::Path};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn ensure(condition: bool, diagnostic: &'static str) -> Result<()> {
    if condition {
        Ok(())
    } else {
        Err(diagnostic.into())
    }
}

fn initialize(path: &Path) -> Result<()> {
    let mut mandate: Mandate = store::read_input()?;
    mandate.validate(store::now()?)?;
    mandate.repository = mandate.repository.canonicalize()?;
    mandate.repository = Path::new(&store::git(
        &mandate.repository,
        &["rev-parse", "--show-toplevel"],
    )?)
    .canonicalize()?;
    let parent = path
        .parent()
        .ok_or("absolute task directory required")?
        .canonicalize()?;
    ensure(
        !parent.starts_with(&mandate.repository),
        "task state must live outside its worktree",
    )?;
    let directory = store::create_directory(path)?;
    store::create_directory(&directory.join("drafts"))?;
    let candidate = Candidate::read(&mandate.repository)?;
    store::save(&directory, &State::new(mandate, candidate))
}

fn execute() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let [operation, directory] = arguments.as_slice() else {
        return Err("usage: agent-supervisor init|run|status|resolve|stop /absolute/private-task; init/resolve read JSON from stdin".into());
    };
    let path = Path::new(directory);
    ensure(path.is_absolute(), "absolute task directory required")?;
    if operation == "init" {
        return initialize(path);
    }
    let directory = store::private_directory(path)?;
    if operation == "stop" {
        match store::create_file(&directory.join("stop")) {
            Ok(file) => file.sync_all()?,
            Err(_) if directory.join("stop").exists() => {}
            Err(error) => return Err(error),
        }
        return Ok(());
    }
    if operation == "status" {
        let state: State = store::read_json(&directory.join("state.json"))?;
        return Ok(writeln!(
            std::io::stdout(),
            "{}",
            serde_json::to_string(&state)?
        )?);
    }
    let _lock = store::lock(&directory.join("lock"))?;
    let mut state: State = store::read_json(&directory.join("state.json"))?;
    match operation.as_str() {
        "run" => supervisor::run(&directory, &mut state)?,
        "resolve" => {
            ensure(
                matches!(
                    state.phase,
                    Phase::WaitingDecision | Phase::WaitingDependency
                ),
                "no resolvable wait",
            )?;
            ensure(
                !directory.join("stop").exists() && store::now()? < state.mandate.expires_at,
                "stopped or expired mandate",
            )?;
            let resolution: Resolution = store::read_input()?;
            ensure(
                resolution.attempt == state.attempts && !resolution.answer.trim().is_empty(),
                "explicit resolution required",
            )?;
            state.resolutions.push(resolution);
            state.phase = Phase::Ready;
            store::save(&directory, &state)?;
        }
        _ => return Err("unknown operation".into()),
    }
    writeln!(std::io::stdout(), "{}", serde_json::to_string(&state)?)?;
    Ok(())
}

fn main() {
    if let Err(error) = execute() {
        let _ = writeln!(std::io::stderr(), "agent-supervisor: {error}");
        std::process::exit(1);
    }
}
