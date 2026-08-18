use clap::{Parser, Subcommand, ValueEnum};
use std::io;
use std::process::ExitCode;

use arnes::Roots;
use arnes::commands;
use arnes::config;
use arnes::diagnostic::{Diagnostic, Report, State};
use arnes::instructions;
use arnes::manifest::{self, Agent, Scope};
use arnes::prompts;
use arnes::skills;

#[derive(Parser)]
#[command(version, about = "Diagnose agent harness resources")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    Doctor {
        #[arg(value_enum)]
        resource: Option<Resource>,
        #[arg(long, value_enum)]
        agent: Option<Agent>,
        #[arg(long, value_enum, default_value = "user")]
        scope: Option<Scope>,
        #[arg(long, value_enum, default_value_t)]
        format: Format,
    },
}

#[derive(Clone, ValueEnum)]
enum Resource {
    Manifest,
    Config,
    Instructions,
    Skills,
    Prompts,
    Commands,
    Rules,
    Hooks,
    Mcp,
    Statusline,
}

#[derive(Clone, Default, ValueEnum)]
enum Format {
    #[default]
    Human,
    Json,
}

fn main() -> ExitCode {
    let Cli { command } = Cli::parse();

    let Command::Doctor {
        resource,
        agent,
        scope,
        format,
    } = command;

    let diagnostics = diagnose(resource, agent, scope);
    let report = Report::new(diagnostics);
    let output = match format {
        Format::Human => report.human(),
        Format::Json => report.json().expect("diagnostics are JSON serializable"),
    };

    if let Err(error) = write_output(&output) {
        eprintln!("output: could not write diagnostics: {error}");
        return ExitCode::from(2);
    }
    ExitCode::from(report.exit_code())
}

fn diagnose(
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    match resource {
        None | Some(Resource::Manifest) => match Roots::from_environment() {
            Ok(roots) => vec![diagnose_manifest(&roots)],
            Err(error) => vec![Diagnostic::new("manifest", State::Error, error.to_string())],
        },
        Some(Resource::Config) => match Roots::from_environment() {
            Ok(roots) => diagnose_config(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("config", State::Error, error.to_string())],
        },
        Some(Resource::Instructions) => match Roots::from_environment() {
            Ok(roots) => diagnose_instructions(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new(
                "instructions",
                State::Error,
                error.to_string(),
            )],
        },
        Some(Resource::Skills) => match Roots::from_environment() {
            Ok(roots) => diagnose_skills(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("skills", State::Error, error.to_string())],
        },
        Some(Resource::Prompts) => match Roots::from_environment() {
            Ok(roots) => diagnose_prompts(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("prompts", State::Error, error.to_string())],
        },
        Some(Resource::Commands) => match Roots::from_environment() {
            Ok(roots) => diagnose_commands(&roots, agent, scope),
            Err(error) => vec![Diagnostic::new("commands", State::Error, error.to_string())],
        },
        _ => Vec::new(),
    }
}

fn write_output(output: &str) -> io::Result<()> {
    if output.is_empty() {
        return Ok(());
    }

    let output = format!("{output}\n");
    let mut remaining = output.as_bytes();
    while !remaining.is_empty() {
        let written =
            rustix::io::write(rustix::stdio::stdout(), remaining).map_err(io::Error::from)?;
        if written == 0 {
            return Err(io::Error::from(io::ErrorKind::WriteZero));
        }
        remaining = &remaining[written..];
    }
    Ok(())
}

fn diagnose_manifest(roots: &Roots) -> Diagnostic {
    match manifest::load(roots.home()) {
        Ok(_) => Diagnostic::new("manifest", State::Healthy, "manifest is valid"),
        Err(error) => Diagnostic::new("manifest", State::Error, error.to_string()),
    }
}

fn diagnose_config(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => config::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("config", State::Error, error.to_string())],
    }
}

fn diagnose_instructions(
    roots: &Roots,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => instructions::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new(
            "instructions",
            State::Error,
            error.to_string(),
        )],
    }
}

fn diagnose_skills(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => skills::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("skills", State::Error, error.to_string())],
    }
}

fn diagnose_prompts(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => prompts::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("prompts", State::Error, error.to_string())],
    }
}

fn diagnose_commands(roots: &Roots, agent: Option<Agent>, scope: Option<Scope>) -> Vec<Diagnostic> {
    match manifest::load(roots.home()) {
        Ok(manifest) => commands::diagnose(roots, &manifest, agent, scope),
        Err(error) => vec![Diagnostic::new("commands", State::Error, error.to_string())],
    }
}
