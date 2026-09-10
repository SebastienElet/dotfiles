use super::contracts::Observation;
use super::fixture::Fixture;
use super::live::Execution;
use super::oracle::evaluate;
use super::process::ExecutionError;
use super::report::{Run, RunError, Status, Tokens};
use super::sources::LoadedCase;

pub fn observe(fixture: &Fixture, entry: &LoadedCase, execution: Execution) -> Run {
    let parsed = std::fs::read_to_string(&fixture.observations)
        .map_err(|error| error.to_string())
        .and_then(|contents| {
            contents
                .lines()
                .filter(|line| !line.is_empty())
                .map(|line| {
                    let observation: Observation =
                        serde_json::from_str(line).map_err(|error| error.to_string())?;
                    if observation.exit_code > 9_007_199_254_740_991 {
                        return Err("Invalid observation exit code".to_owned());
                    }
                    Ok(observation)
                })
                .collect::<Result<Vec<_>, _>>()
        });
    let (observations, error) = match parsed {
        Ok(observations) => (observations, execution.error.map(run_error)),
        Err(_) => (Vec::new(), Some(RunError::ObservationInvalid)),
    };
    Run {
        status: if error.is_some() {
            Status::Invalid
        } else {
            evaluate(entry.definition.oracle, &observations)
        },
        error,
        observations,
        tokens: execution.tokens.map(|tokens| Tokens {
            input: tokens.input,
            cached_input: tokens.cached_input,
            output: tokens.output,
        }),
        tool_calls: execution.tool_calls,
        duration_ms: execution.duration_ms,
    }
}

const fn run_error(error: ExecutionError) -> RunError {
    match error {
        ExecutionError::AgentFailed => RunError::AgentFailed,
        ExecutionError::Timeout => RunError::Timeout,
        ExecutionError::OutputLimit => RunError::OutputLimit,
        ExecutionError::ProtocolInvalid => RunError::ProtocolInvalid,
    }
}
