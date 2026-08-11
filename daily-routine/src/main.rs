#[allow(
    dead_code,
    reason = "Bitbucket collection is consumed by later provider orchestration"
)]
mod bitbucket;
#[allow(
    dead_code,
    reason = "CLI execution is consumed by later provider orchestration"
)]
mod command;
#[allow(
    dead_code,
    reason = "configuration scope is consumed by later orchestration"
)]
mod config;
#[allow(
    dead_code,
    reason = "Linear collection is consumed by later provider orchestration"
)]
mod linear;
#[allow(
    dead_code,
    reason = "normalized work is consumed by later orchestration"
)]
mod model;
#[allow(
    dead_code,
    reason = "bounded detail mapping is consumed by later provider orchestration"
)]
mod parallel;
#[allow(
    dead_code,
    reason = "track selection is consumed by later orchestration"
)]
mod rules;
#[allow(
    dead_code,
    reason = "standard-library utilities are consumed by later orchestration"
)]
mod util;

use config::Config;
use std::error::Error;
use std::process::ExitCode;

const CONFIG_EXAMPLE: &str = include_str!("../config.example.toml");

fn config_error_message(error: &dyn Error) -> String {
    format!("failed to load configuration: {error}\n\n{CONFIG_EXAMPLE}")
}

fn main() -> ExitCode {
    match Config::load() {
        Ok(_) => ExitCode::SUCCESS,
        Err(error) => {
            eprint!("{}", config_error_message(error.as_ref()));
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_error_message_includes_diagnostic_and_complete_example() {
        let error = std::io::Error::other("invalid configuration");

        let message = config_error_message(&error);

        assert_eq!(
            message,
            format!(
                "failed to load configuration: {error}\n\n{}",
                include_str!("../config.example.toml")
            )
        );
    }
}
