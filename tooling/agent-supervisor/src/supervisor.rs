use std::{path::Path, thread, time::Duration};

use crate::{
    Result, codex, ensure,
    model::{Candidate, Outcome, Phase, State, Submission},
    process::Cancellation,
    store,
};

fn halt(directory: &Path, state: &mut State, cancellation: &Cancellation) -> Result<bool> {
    let phase = if cancellation.stopped(directory) {
        Some(Phase::Stopped)
    } else if store::now()? >= state.mandate.expires_at {
        Some(Phase::Expired)
    } else {
        None
    };
    if let Some(phase) = phase {
        state.phase = phase;
        store::save(directory, state)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn run(directory: &Path, state: &mut State) -> Result<()> {
    let cancellation = Cancellation::new()?;
    if halt(directory, state, &cancellation)? {
        return Ok(());
    }
    if matches!(state.phase, Phase::Dispatching | Phase::Submitted) {
        state.phase = Phase::Unconfirmed;
        state.diagnostic = Some("interrupted submission; inspect before any new mandate".into());
        return store::save(directory, state);
    }
    if !matches!(state.phase, Phase::Ready | Phase::WaitingCapacity) {
        return Ok(());
    }
    let repository = state
        .mandate
        .repository
        .canonicalize()
        .map_err(|_| "candidate repository unavailable")?;
    let git_directory = store::git(&repository, &["rev-parse", "--absolute-git-dir"])?;
    let _worktree_lock = store::lock(&Path::new(&git_directory).join("agent-supervisor.lock"))?;
    loop {
        if halt(directory, state, &cancellation)? {
            return Ok(());
        }
        if state.attempts >= state.mandate.max_turns {
            state.phase = Phase::BudgetExhausted;
            return store::save(directory, state);
        }
        while store::now()? < state.retry_at {
            if halt(directory, state, &cancellation)? {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(100));
        }
        if !await_capacity(directory, state, &cancellation)? {
            if state.phase == Phase::WaitingCapacity {
                continue;
            }
            return Ok(());
        }
        if halt(directory, state, &cancellation)? {
            return Ok(());
        }
        ensure(
            Candidate::read(&repository)? == state.candidate,
            "candidate branch or HEAD changed outside the supervised turn",
        )?;
        state.attempts += 1;
        state.receipts.push(Submission::new(state.attempts));
        state.phase = Phase::Dispatching;
        state.diagnostic = None;
        store::save(directory, state)?;
        match codex::turn(directory, state, &cancellation) {
            Ok(checkpoint) => {
                state.phase = match checkpoint.outcome {
                    Outcome::Handoff => Phase::Ready,
                    Outcome::Decision => Phase::WaitingDecision,
                    Outcome::Dependency => Phase::WaitingDependency,
                    Outcome::Complete => Phase::Complete,
                };
                state.checkpoint = Some(checkpoint);
                match Candidate::read(&repository) {
                    Ok(candidate) => state.candidate = candidate,
                    Err(_) => {
                        state.diagnostic = Some(
                            "candidate unavailable; checkpoint retained in private task state"
                                .into(),
                        )
                    }
                }
            }
            Err(error) => {
                state.phase = Phase::Unconfirmed;
                state.diagnostic = Some(error.to_string());
            }
        }
        store::save(directory, state)?;
        if halt(directory, state, &cancellation)? || state.phase != Phase::Ready {
            return Ok(());
        }
    }
}

fn await_capacity(
    directory: &Path,
    state: &mut State,
    cancellation: &Cancellation,
) -> Result<bool> {
    if state.capacity_probes >= state.mandate.max_capacity_probes {
        state.phase = Phase::BudgetExhausted;
        store::save(directory, state)?;
        return Ok(false);
    }
    state.capacity_probes += 1;
    state.phase = Phase::WaitingCapacity;
    state.retry_at = store::now()?
        .checked_add(state.mandate.retry_seconds)
        .ok_or("retry overflow")?;
    store::save(directory, state)?;
    match codex::capacity(directory, state, cancellation) {
        Ok(true) => {
            state.retry_at = 0;
            Ok(true)
        }
        result => {
            state.diagnostic = Some(result.err().map_or_else(
                || "provider capacity exhausted".into(),
                |error| error.to_string(),
            ));
            store::save(directory, state)?;
            Ok(false)
        }
    }
}
