use arnes::measure;
use std::process::ExitCode;

use crate::cli::MeasureCommand;
use crate::cli_output::write_output;

pub fn run_measure(command: MeasureCommand) -> ExitCode {
    match command {
        MeasureCommand::Hook { agent } => match measure::capture(agent) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                let _ = crate::cli_output::write_error(format_args!("measure hook: {error}"));
                ExitCode::SUCCESS
            }
        },
        MeasureCommand::List(args) => match measure::list(args) {
            Ok(output) => match write_output(&output) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => fail_measure(error),
            },
            Err(error) => fail_measure(error),
        },
        MeasureCommand::Finish(args) => finish_measure(measure::finish(args)),
        MeasureCommand::Feedback(args) => finish_measure(measure::feedback(args)),
        MeasureCommand::Outcome(args) => finish_measure(measure::outcome(args)),
        MeasureCommand::PrVerdict(args) => finish_measure(
            measure::pr_verdict(args).and_then(|status| write_output(status).map_err(Into::into)),
        ),
        MeasureCommand::Report(args) => match measure::report(args) {
            Ok(output) => match write_output(&output) {
                Ok(()) => ExitCode::SUCCESS,
                Err(error) => fail_measure(error),
            },
            Err(error) => fail_measure(error),
        },
    }
}

fn finish_measure(result: Result<(), measure::MeasureError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail_measure(error),
    }
}

fn fail_measure(error: impl std::fmt::Display) -> ExitCode {
    let _ = crate::cli_output::write_error(format_args!("measure: {error}"));
    ExitCode::from(2)
}
