use serde::Deserialize;
use std::{
    cell::RefCell,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    Result, ensure, handoff,
    model::{self, Checkpoint, Outcome, Phase, State},
    process::{self, Cancellation},
    store,
};

#[derive(Deserialize)]
struct Event {
    r#type: String,
    thread_id: Option<String>,
}

#[derive(Default)]
struct Receipt {
    pending: Vec<u8>,
    thread_id: Option<String>,
    started: bool,
    completed: bool,
    failed: bool,
    active: bool,
    turns_started: u32,
}

impl Receipt {
    fn consume(&mut self, bytes: &[u8]) -> Result<()> {
        self.pending.extend_from_slice(bytes);
        ensure(
            self.pending.len() as u64 <= store::MAX_BYTES,
            "oversized event",
        )?;
        while let Some(end) = self.pending.iter().position(|byte| *byte == b'\n') {
            let line: Vec<u8> = self.pending.drain(..=end).collect();
            let event: Event = serde_json::from_slice(&line).map_err(|_| "invalid event stream")?;
            match event.r#type.as_str() {
                "thread.started" => {
                    ensure(
                        self.thread_id.is_none() && !self.started,
                        "unexpected thread event",
                    )?;
                    self.thread_id = event.thread_id.filter(|id| !id.is_empty());
                }
                "turn.started" => {
                    ensure(
                        self.thread_id.is_some() && !self.active,
                        "unattributed turn",
                    )?;
                    self.started = true;
                    self.completed = false;
                    self.active = true;
                    self.turns_started = self
                        .turns_started
                        .checked_add(1)
                        .ok_or("turn count overflow")?;
                }
                "turn.completed" => {
                    ensure(self.active, "unexpected completion")?;
                    self.completed = true;
                    self.active = false;
                }
                "turn.failed" | "error" => self.failed = true,
                _ => {}
            }
        }
        Ok(())
    }

    fn persist(&self, directory: &Path, state: &mut State) -> Result<()> {
        let fresh_thread = self.thread_id.as_ref().is_some_and(|id| {
            !state
                .receipts
                .iter()
                .take(state.receipts.len().saturating_sub(1))
                .any(|proof| proof.thread_id.as_ref() == Some(id))
        });
        let proof = state.receipts.last_mut().ok_or("missing submission")?;
        if proof.thread_id == self.thread_id
            && proof.started == self.started
            && proof.completed == self.completed
            && proof.turns_started == self.turns_started
        {
            return Ok(());
        }
        proof.thread_id = self.thread_id.clone();
        proof.fresh_thread = fresh_thread;
        proof.started = self.started;
        proof.completed = self.completed;
        proof.turns_started = self.turns_started;
        store::save(directory, state)
    }
}

pub fn turn(
    directory: &Path,
    state: &mut State,
    cancellation: &Cancellation,
) -> Result<Checkpoint> {
    let mut attempt = prepare(directory, state, cancellation)?;
    let mut receipt = Receipt::default();
    let timeout = state
        .mandate
        .turn_timeout_seconds
        .min(state.mandate.expires_at.saturating_sub(store::now()?));
    let expires_at = state.mandate.expires_at;
    let shared = RefCell::new(state);
    process::run(
        &mut attempt.command,
        directory,
        cancellation,
        Duration::from_secs(timeout),
        expires_at,
        |bytes| {
            let result = receipt.consume(bytes);
            receipt.persist(directory, &mut shared.borrow_mut())?;
            result
        },
        |process_group| {
            let mut state = shared.borrow_mut();
            state.phase = Phase::Submitted;
            let proof = state.receipts.last_mut().ok_or("missing submission")?;
            proof.accepted = true;
            proof.process_group = Some(process_group);
            store::save(directory, &state)
        },
    )?;
    let state = shared.into_inner();
    ensure(
        receipt.pending.is_empty() && receipt.started && receipt.completed && !receipt.failed,
        "submission unconfirmed or turn failed; no automatic resend",
    )?;
    ensure(
        state
            .receipts
            .last()
            .is_some_and(|proof| proof.fresh_thread),
        "old thread cannot confirm a new attempt",
    )?;
    let has_note = attempt.note.exists();
    let path = if has_note {
        &attempt.note
    } else {
        &attempt.reply
    };
    let checkpoint = handoff::checkpoint(path, directory, state, cancellation)?;
    ensure(
        checkpoint.outcome != Outcome::Handoff || has_note,
        "handoff requires a published durable note",
    )?;
    Ok(checkpoint)
}

struct Attempt {
    command: Command,
    note: PathBuf,
    reply: PathBuf,
}

fn prepare(directory: &Path, state: &State, cancellation: &Cancellation) -> Result<Attempt> {
    let attempt = store::create_directory(&directory.join(format!("attempt-{}", state.attempts)))?;
    let notes = store::create_directory(&attempt.join("notes"))?;
    let note = notes.join("note.json");
    let reply = attempt.join("result.json");
    store::write_json(
        &attempt.join("schema.json"),
        &handoff::schema(directory, state, cancellation)?,
    )?;
    store::write_json(&attempt.join("context.json"), &handoff::context(state))?;
    let prompt_path = attempt.join("prompt");
    use std::io::Write;
    store::create_file(&prompt_path)?.write_all(
        model::prompt(
            state,
            &directory.join("drafts"),
            &handoff::skill_path(directory, state, cancellation)?,
        )?
        .as_bytes(),
    )?;
    let mut command = Command::new(&state.mandate.codex);
    command
        .args([
            "--ask-for-approval",
            "never",
            "exec",
            "--config",
            "model_provider=\"openai\"",
            "--json",
            "--color",
            "never",
            "--model",
            &state.mandate.model,
            "--sandbox",
            state.mandate.sandbox.argument(),
            "--cd",
        ])
        .arg(&state.mandate.repository)
        .arg("--add-dir")
        .arg(directory.join("drafts"))
        .arg("--add-dir")
        .arg(notes)
        .arg("--output-schema")
        .arg(attempt.join("schema.json"))
        .arg("--output-last-message")
        .arg(&reply)
        .arg("-")
        .stdin(fs::File::open(&prompt_path)?)
        .current_dir(&state.mandate.repository)
        .env("HANDOFF_NOTE_PATH", &note)
        .env("HANDOFF_CONTEXT_PATH", attempt.join("context.json"))
        .env("HANDOFF_EXECUTABLE", &state.mandate.handoff);
    Ok(Attempt {
        command,
        note,
        reply,
    })
}

#[derive(Deserialize)]
struct Capacity {
    provider: String,
    source: String,
    error: Option<serde_json::Value>,
    usage: Usage,
}
#[derive(Deserialize)]
struct Usage {
    primary: Option<Window>,
    secondary: Option<Window>,
}
#[derive(Deserialize)]
struct Window {
    #[serde(rename = "usedPercent")]
    used_percent: f64,
    #[serde(default, rename = "isSyntheticPlaceholder")]
    synthetic: bool,
}

pub fn capacity(directory: &Path, state: &State, cancellation: &Cancellation) -> Result<bool> {
    let mut command = Command::new(&state.mandate.codexbar);
    command
        .args(["usage", "--provider", "codex", "--source", "cli", "--json"])
        .stdin(Stdio::null())
        .current_dir(&state.mandate.repository);
    let mut output = Vec::new();
    let timeout = 10.min(state.mandate.expires_at.saturating_sub(store::now()?));
    process::run(
        &mut command,
        directory,
        cancellation,
        Duration::from_secs(timeout),
        state.mandate.expires_at,
        |bytes| {
            output.extend_from_slice(bytes);
            ensure(output.len() <= 65536, "capacity response too large")
        },
        |_| Ok(()),
    )?;
    let capacities: Vec<Capacity> =
        serde_json::from_slice(&output).map_err(|_| "capacity unavailable")?;
    ensure(
        capacities.len() == 1,
        "capacity provider ambiguous or absent",
    )?;
    let capacity = capacities.first().ok_or("capacity absent")?;
    ensure(
        capacity.provider == "codex" && capacity.source == "codex-cli" && capacity.error.is_none(),
        "capacity identity or source invalid",
    )?;
    let values: Vec<f64> = [
        capacity.usage.primary.as_ref(),
        capacity.usage.secondary.as_ref(),
    ]
    .into_iter()
    .flatten()
    .filter(|window| !window.synthetic)
    .map(|window| window.used_percent)
    .collect();
    ensure(!values.is_empty(), "capacity measurement absent")?;
    ensure(
        values
            .iter()
            .all(|value| value.is_finite() && (0.0..=100.0).contains(value)),
        "invalid capacity",
    )?;
    Ok(values.iter().all(|value| *value < 90.0))
}
