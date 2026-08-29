use crate::{
    Environment, HandoffError, SentinelState, create_sentinel, find_latest_usage, handoff_output,
    inspect_sentinel, parse_hook_event, select_threshold, state_root,
};
use std::fs;
use std::io::Write;

pub fn run_agent_handoff(
    input: &[u8],
    environment: &Environment,
    stdout: &mut impl Write,
) -> Result<(), HandoffError> {
    let event = parse_hook_event(input)?;
    if event.stop_hook_active {
        return Ok(());
    }

    let sentinel = state_root(environment)?
        .join("dotfiles")
        .join("handoff")
        .join(event.session_id);
    if inspect_sentinel(&sentinel)? {
        return Ok(());
    }

    let transcript = fs::read_to_string(event.transcript_path)
        .map_err(|_| HandoffError::usage("cannot read transcript"))?;
    let usage = find_latest_usage(&transcript)?;
    let threshold = select_threshold(&usage, environment)?;
    if usage.used < threshold {
        return Ok(());
    }
    if create_sentinel(&sentinel)? == SentinelState::Existing {
        return Ok(());
    }

    stdout
        .write_all(&handoff_output(&usage, threshold))
        .map_err(|_| HandoffError::unexpected("unexpected failure"))
}
