use clap::{Parser, Subcommand, ValueEnum};

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

    match command {
        Command::Doctor {
            format: Format::Json,
            ..
        } => println!("[]"),
        Command::Doctor { .. } => {}
    }
}
