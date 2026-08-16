use clap::{Parser, Subcommand, ValueEnum};
use std::io;
use std::process::ExitCode;

use arnes::Roots;
use arnes::diagnostic::{Diagnostic, Report, State};
use arnes::manifest;

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
        #[arg(long, value_enum)]
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

#[derive(Clone, ValueEnum)]
enum Agent {
    Claude,
    Cursor,
    Codex,
}

#[derive(Clone, ValueEnum)]
enum Scope {
    User,
    Project,
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
        resource, format, ..
    } = command;

    let diagnostics = if matches!(resource, None | Some(Resource::Manifest)) {
        let diagnostic = match Roots::from_environment() {
            Ok(roots) => diagnose_manifest(&roots),
            Err(error) => Diagnostic::new("manifest", State::Error, error.to_string()),
        };
        vec![diagnostic]
    } else {
        Vec::new()
    };
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
