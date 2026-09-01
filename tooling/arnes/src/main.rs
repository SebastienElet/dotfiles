mod cli;
mod cli_output;
mod doctor;
mod measure_cli;

use clap::Parser;
use std::process::ExitCode;

use arnes::Roots;
use arnes::export;
use arnes::hooks;
use cli::{Cli, Command, SetupCommand};
use measure_cli::run_measure;

fn main() -> ExitCode {
    let Cli { command } = Cli::parse();

    match command {
        Command::Export { check } => run_export(check),
        Command::Doctor {
            resource,
            agent,
            scope,
            format,
            color,
            verbose,
        } => doctor::run(resource, agent, scope, format, color, verbose),
        Command::Measure { command } => run_measure(command),
        Command::Setup { command } => run_setup(command),
    }
}

fn run_export(check: bool) -> ExitCode {
    let result = Roots::from_environment()
        .map_err(|error| error.to_string())
        .and_then(|roots| export::run(&roots, check).map_err(|error| error.to_string()));
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("export: {error}");
            ExitCode::from(2)
        }
    }
}

fn run_setup(command: SetupCommand) -> ExitCode {
    let result = match command {
        SetupCommand::Hooks(args) => hooks::setup(args),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("setup: {error}");
            ExitCode::from(2)
        }
    }
}
