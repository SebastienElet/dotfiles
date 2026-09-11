use crate::state::join_posix;
use crate::{
    Environment, HandoffError, SentinelState, create_sentinel, find_latest_usage, handoff_output,
    inspect_sentinel, parse_hook_event, select_threshold, state_root,
};
use std::ffi::OsStr;
use std::fs;
use std::io::Write;

/// # Errors
/// Returns usage errors for invalid input, transcript, or threshold; returns unexpected errors for sentinel or output failures.
pub fn run_agent_handoff(
    input: &[u8],
    environment: &Environment,
    stdout: &mut impl Write,
) -> Result<(), HandoffError> {
    let event = parse_hook_event(input)?;
    if event.stop_hook_active {
        return Ok(());
    }

    let state_root = state_root(environment)?;
    let sentinel = join_posix(&[
        state_root.as_os_str(),
        OsStr::new("dotfiles"),
        OsStr::new("handoff"),
        OsStr::new(&event.session_id),
    ]);
    if inspect_sentinel(&sentinel)? {
        return Ok(());
    }

    let transcript = fs::read(event.transcript_path)
        .map_err(|_| HandoffError::usage("cannot read transcript"))?;
    let transcript = String::from_utf8_lossy(&transcript);
    let Some(usage) = find_latest_usage(&transcript)? else {
        return Ok(());
    };
    let threshold = select_threshold(&usage, environment)?;
    if usage.used < threshold {
        return Ok(());
    }
    let output = handoff_output(&usage, threshold)?;
    if create_sentinel(&sentinel)? == SentinelState::Existing {
        return Ok(());
    }

    stdout
        .write_all(&output)
        .map_err(|_| HandoffError::unexpected("unexpected failure"))
}
