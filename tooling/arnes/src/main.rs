mod cli;

use clap::Parser;
use std::io::{self, IsTerminal};
use std::process::ExitCode;

use arnes::Roots;
use arnes::commands;
use arnes::config;
use arnes::diagnostic::{ColorMode, Diagnostic, HumanContext, HumanOptions, Report, State};
use arnes::instructions;
use arnes::manifest::{self, Agent, Scope};
use arnes::measure;
use arnes::prompts;
use arnes::skills;
use cli::{Cli, Color, Command, Format, MeasureCommand, Resource, validate_render_options};

fn main() -> ExitCode {
    let Cli { command } = Cli::parse();

    match command {
        Command::Doctor {
            resource,
            agent,
            scope,
            format,
            color,
            verbose,
        } => run_doctor(resource, agent, scope, format, color, verbose),
        Command::Measure { command } => run_measure(command),
    }
}

fn run_measure(command: MeasureCommand) -> ExitCode {
    match command {
        MeasureCommand::Hook { agent } => match measure::capture(agent) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("measure hook: {error}");
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
    }
}

fn finish_measure(result: Result<(), measure::MeasureError>) -> ExitCode {
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => fail_measure(error),
    }
}

fn fail_measure(error: impl std::fmt::Display) -> ExitCode {
    eprintln!("measure: {error}");
    ExitCode::from(2)
}

fn run_doctor(
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
    format: Format,
    color: Color,
    verbose: bool,
) -> ExitCode {
    if let Err(error) = validate_render_options(format, verbose, color) {
        eprintln!("{error}");
        return ExitCode::from(2);
    }

    let diagnostics = diagnose(resource, agent, scope);
    let report = Report::new(diagnostics);
    let output = match format {
        Format::Human => report.human(
            &human_context(resource, agent, scope),
            (if verbose {
                HumanOptions::verbose()
            } else {
                HumanOptions::normal()
            })
            .with_color(
                ColorMode::from(color),
                io::stdout().is_terminal(),
                std::env::var_os("NO_COLOR").as_deref(),
            ),
        ),
        Format::Json => report.json().expect("diagnostics are JSON serializable"),
    };

    if let Err(error) = write_output(&output) {
        eprintln!("output: could not write diagnostics: {error}");
        return ExitCode::from(2);
    }
    ExitCode::from(report.exit_code())
}

impl Resource {
    fn heading(self) -> &'static str {
        match self {
            Self::Manifest => "Manifest",
            Self::Config => "Config",
            Self::Instructions => "Instructions",
            Self::Skills => "Skills",
            Self::Prompts => "Prompts",
            Self::Commands => "Commands",
            Self::Rules => "Rules",
            Self::Hooks => "Hooks",
            Self::Mcp => "MCP",
            Self::Statusline => "Statusline",
        }
    }
}

fn human_context(
    resource: Option<Resource>,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> HumanContext {
    let resource = resource.unwrap_or(Resource::Manifest);
    let mut context = HumanContext::new(resource.heading());
    if resource != Resource::Manifest {
        if let Some(scope) = scope {
            context = context.with_qualifier(format!("{scope} scope"));
        }
        if let Some(agent) = agent {
            context = context.with_qualifier(format!("{agent} agent"));
        } else if resource == Resource::Skills {
            context = context.with_section_count("agent", "agents", "all agents");
        }
    }
    context
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
