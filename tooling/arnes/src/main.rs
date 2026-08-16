use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

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

fn main() {
    let Cli { command } = Cli::parse();

    let Command::Doctor {
        resource, format, ..
    } = command;

    if matches!(resource, None | Some(Resource::Manifest)) {
        let Some(home) = std::env::var_os("HOME") else {
            eprintln!("HOME: environment variable is required");
            std::process::exit(1);
        };
        if let Err(error) = manifest::load(&PathBuf::from(home)) {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }

    if matches!(format, Format::Json) {
        println!("[]");
    }
}
