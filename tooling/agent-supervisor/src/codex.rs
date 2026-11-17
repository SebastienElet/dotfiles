use serde::Deserialize;
use std::{
    cell::RefCell,
    fs,
    path::Path,
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    Result, ensure,
    model::{self, Checkpoint, Phase, State},
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
                        self.thread_id.is_some() && !self.started,
                        "unattributed turn",
                    )?;
                    self.started = true;
                }
                "turn.completed" => {
                    ensure(self.started && !self.completed, "unexpected completion")?;
                    self.completed = true;
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
        {
            return Ok(());
        }
        proof.thread_id = self.thread_id.clone();
        proof.fresh_thread = fresh_thread;
        proof.started = self.started;
        proof.completed = self.completed;
        store::save(directory, state)
    }
}

pub fn turn(
    directory: &Path,
    state: &mut State,
    cancellation: &Cancellation,
) -> Result<Checkpoint> {
    let attempt = store::create_directory(&directory.join(format!("attempt-{}", state.attempts)))?;
    store::write_json(&attempt.join("schema.json"), &model::checkpoint_schema())?;
    let prompt_path = attempt.join("prompt");
    use std::io::Write;
    store::create_file(&prompt_path)?
        .write_all(model::prompt(state, &directory.join("drafts"))?.as_bytes())?;
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
            "--ephemeral",
            "--model",
            &state.mandate.model,
            "--sandbox",
            state.mandate.sandbox.argument(),
            "--cd",
        ])
        .arg(&state.mandate.repository)
        .arg("--add-dir")
        .arg(directory.join("drafts"))
        .arg("--output-schema")
        .arg(attempt.join("schema.json"))
        .arg("--output-last-message")
        .arg(attempt.join("result.json"))
        .arg("-")
        .stdin(fs::File::open(&prompt_path)?)
        .current_dir(&state.mandate.repository);
    let mut receipt = Receipt::default();
    let timeout = state
        .mandate
        .turn_timeout_seconds
        .min(state.mandate.expires_at.saturating_sub(store::now()?));
    let expires_at = state.mandate.expires_at;
    let shared = RefCell::new(state);
    process::run(
        &mut command,
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
    let checkpoint: Checkpoint = store::read_json(&attempt.join("result.json"))?;
    checkpoint.validate()?;
    Ok(checkpoint)
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
