mod bitbucket;
mod core;
mod linear;
mod model;
mod process;
mod providers;
mod run;
mod writes;

use clap::Parser;
use model::{Config, Fallible};
use std::{
    io::{self, Write},
    path::PathBuf,
    process::ExitCode,
};

#[derive(Parser)]
#[command(version, about = "Reconcile Bitbucket Cloud pull requests with Linear issues", after_help = include_str!("help.txt"))]
struct Options {
    #[arg(long)]
    config: PathBuf,
    #[arg(long)]
    apply: bool,
    #[arg(long)]
    json: bool,
}

fn configuration(options: &Options) -> Fallible<Config> {
    let bytes = std::fs::read(&options.config).map_err(|_| "configuration unreadable")?;
    let config: Config = serde_json::from_slice(&bytes).map_err(|_| "configuration invalid")?;
    config.validate()?;
    Ok(config)
}

fn main() -> ExitCode {
    let options = match Options::try_parse() {
        Ok(options) => options,
        Err(error) => {
            if error.use_stderr() {
                let _ = writeln!(
                    io::stderr().lock(),
                    "bitbucket-linear-sync: usage/configuration invalid; use --help"
                );
                return ExitCode::from(2);
            }
            return if error.print().is_ok() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            };
        }
    };
    let config = match configuration(&options) {
        Ok(config) => config,
        Err(reason) => {
            let _ = writeln!(
                io::stderr().lock(),
                "bitbucket-linear-sync: usage/configuration invalid ({reason}); use --help"
            );
            return ExitCode::from(2);
        }
    };
    match execute(&options, &config) {
        Ok(code) => ExitCode::from(code),
        Err(reason) => {
            let _ = writeln!(
                io::stderr().lock(),
                "bitbucket-linear-sync: provider/inventory/output failed ({reason})"
            );
            ExitCode::FAILURE
        }
    }
}

fn execute(options: &Options, config: &Config) -> Fallible<u8> {
    let providers = providers::Providers { config };
    let plan = run::collect(&providers)?;
    let mut output = io::stdout().lock();
    let public = plan.public();
    if options.json {
        writeln!(output, "{public}")
    } else {
        serde_json::to_string_pretty(&public)
            .map_err(|_| "plan serialization failed")
            .and_then(|text| writeln!(output, "Plan\n{text}").map_err(|_| "plan output failed"))
            .map_err(io::Error::other)
    }
    .map_err(|_| "plan output failed")?;
    output.flush().map_err(|_| "plan output failed")?;
    let (observations, code) = run::finish(plan, &providers, options.apply);
    if options.json {
        let mode = if options.apply { "apply" } else { "inspect" };
        writeln!(output, "{}", serde_json::json!({"phase":"result","mode":mode,"observations":observations,"exitCode":code})).map_err(|_| "result output failed")?;
    } else {
        for event in observations {
            let identity = event
                .identifier
                .as_deref()
                .or(event.repository.as_deref())
                .or(event.url.as_deref())
                .unwrap_or("inventory");
            let reason = event.reason.map_or_else(String::new, |r| format!(" ({r})"));
            writeln!(output, "{}: {identity}{reason}", event.status.text())
                .map_err(|_| "result output failed")?;
        }
    }
    Ok(code)
}

fn ci_420_intentional_rust_fault( {
