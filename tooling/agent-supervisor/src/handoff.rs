use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};

use crate::{
    Result, ensure,
    model::{Checkpoint, State},
    process::{self, Cancellation},
    store,
};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Note {
    format_version: u8,
    task_id: String,
    owner: String,
    goal: String,
    candidate: NoteCandidate,
    permissions: Vec<String>,
    checkpoint: Checkpoint,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct NoteCandidate {
    repository: PathBuf,
    head: String,
    branch: String,
}

pub fn context(state: &State) -> Value {
    json!({"format_version":1,"task_id":state.mandate.task_id,"owner":state.mandate.owner,
        "goal":state.mandate.goal,"candidate":{"repository":state.mandate.repository,
        "head":state.candidate.head,"branch":state.candidate.branch},"permissions":state.mandate.permissions})
}

fn output(
    directory: &Path,
    state: &State,
    cancellation: &Cancellation,
    command: &mut Command,
) -> Result<Vec<u8>> {
    let mut output = Vec::new();
    process::run(
        command,
        directory,
        cancellation,
        Duration::from_secs(10),
        state.mandate.expires_at,
        |bytes| {
            output.extend_from_slice(bytes);
            ensure(
                output.len() as u64 <= store::MAX_BYTES,
                "handoff response too large",
            )
        },
        |_| Ok(()),
    )?;
    Ok(output)
}

pub fn schema(directory: &Path, state: &State, cancellation: &Cancellation) -> Result<Value> {
    let mut command = Command::new(&state.mandate.handoff);
    command
        .arg("note-schema")
        .stdin(Stdio::null())
        .current_dir(directory);
    serde_json::from_slice(&output(directory, state, cancellation, &mut command)?)
        .map_err(|_| "handoff schema unavailable".into())
}

pub fn skill_path(directory: &Path, state: &State, cancellation: &Cancellation) -> Result<PathBuf> {
    let mut command = Command::new(&state.mandate.handoff);
    command
        .arg("skill-path")
        .stdin(Stdio::null())
        .current_dir(directory);
    let path = String::from_utf8(output(directory, state, cancellation, &mut command)?)?;
    let path = PathBuf::from(path.trim());
    ensure(
        path.is_absolute() && path.is_file(),
        "handoff skill unavailable",
    )?;
    Ok(path)
}

pub fn checkpoint(
    path: &Path,
    directory: &Path,
    state: &State,
    cancellation: &Cancellation,
) -> Result<Checkpoint> {
    let metadata = std::fs::symlink_metadata(path)?;
    ensure(
        metadata.is_file()
            && !metadata.file_type().is_symlink()
            && metadata.len() <= store::MAX_BYTES,
        "expected a bounded regular handoff note",
    )?;
    let mut command = Command::new(&state.mandate.handoff);
    command
        .arg("note-validate")
        .stdin(File::open(path)?)
        .current_dir(directory);
    let note: Note = serde_json::from_slice(&output(directory, state, cancellation, &mut command)?)
        .map_err(|_| "invalid normalized handoff note")?;
    ensure(
        note.format_version == 1
            && note.task_id == state.mandate.task_id
            && note.owner == state.mandate.owner
            && note.goal == state.mandate.goal
            && note.permissions == state.mandate.permissions
            && note.candidate.repository == state.mandate.repository
            && note.candidate.head == state.candidate.head
            && note.candidate.branch == state.candidate.branch,
        "handoff note does not match the current mandate",
    )?;
    Ok(note.checkpoint)
}
