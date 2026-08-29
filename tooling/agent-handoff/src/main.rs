use agent_handoff::{Environment, HandoffError, run_agent_handoff};
use std::io::{self, Read, Write};
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let _ = writeln!(io::stderr().lock(), "agent-handoff: {}", error.message);
            ExitCode::from(error.exit_code)
        }
    }
}

fn run() -> Result<(), HandoffError> {
    let mut input = Vec::new();
    io::stdin()
        .lock()
        .read_to_end(&mut input)
        .map_err(|_| HandoffError::unexpected("unexpected failure"))?;
    run_agent_handoff(&input, &Environment::current(), &mut io::stdout().lock())
}
