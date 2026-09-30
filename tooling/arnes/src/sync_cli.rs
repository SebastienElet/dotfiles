use crate::cli::Format;
use arnes::Roots;
use arnes::manifest::{self, Agent, Scope};
use arnes::sync::{self, SyncResource};
use clap::Args;
use std::process::ExitCode;

#[derive(Args, Clone, Copy)]
pub struct SyncArgs {
    #[arg(value_enum)]
    resource: SyncResource,
    #[arg(long, value_enum)]
    agent: Agent,
    #[arg(long, value_enum)]
    scope: Scope,
    #[arg(long, value_enum, default_value_t)]
    format: Format,
}

pub fn run(args: SyncArgs) -> ExitCode {
    let result = Roots::from_environment()
        .map_err(|error| error.to_string())
        .and_then(|roots| {
            let manifest = manifest::load(roots.home()).map_err(|error| error.to_string())?;
            Ok(sync::run(
                &roots,
                &manifest,
                args.resource,
                args.agent,
                args.scope,
            ))
        });
    let report = match result {
        Ok(report) => report,
        Err(error) => {
            let _ = crate::cli_output::write_error(format_args!("sync: {error}"));
            return ExitCode::from(2);
        }
    };
    let output = match args.format {
        Format::Human => Ok(report.human()),
        Format::Json => serde_json::to_string_pretty(&report),
    };
    match output {
        Ok(output) if crate::cli_output::write_output(&output).is_ok() => {
            ExitCode::from(report.exit_code())
        }
        Ok(_) | Err(_) => ExitCode::from(2),
    }
}
