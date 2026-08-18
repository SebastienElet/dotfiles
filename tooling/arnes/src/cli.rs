use arnes::diagnostic::ColorMode;
use arnes::manifest::{Agent, Scope};
use arnes::measure::{FeedbackArgs, FinishArgs, HookAgent, ListArgs};
use clap::{Parser, Subcommand, ValueEnum};

#[derive(Parser)]
#[command(version, about = "Diagnose agent harness resources")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    Doctor {
        #[arg(value_enum)]
        resource: Option<Resource>,
        #[arg(long, value_enum)]
        agent: Option<Agent>,
        #[arg(long, value_enum, default_value = "user")]
        scope: Option<Scope>,
        #[arg(long, value_enum, default_value_t)]
        format: Format,
        #[arg(long, value_enum, default_value_t)]
        color: Color,
        #[arg(short, long)]
        verbose: bool,
    },
    Measure {
        #[command(subcommand)]
        command: MeasureCommand,
    },
}

#[derive(Subcommand)]
pub(crate) enum MeasureCommand {
    Hook {
        #[arg(long, value_enum)]
        agent: HookAgent,
    },
    List(ListArgs),
    Finish(FinishArgs),
    Feedback(FeedbackArgs),
}

#[derive(Clone, Copy, Eq, PartialEq, ValueEnum)]
pub(crate) enum Resource {
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

#[derive(Clone, Copy, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum Format {
    #[default]
    Human,
    Json,
}

#[derive(Clone, Copy, Default, Eq, PartialEq, ValueEnum)]
pub(crate) enum Color {
    #[default]
    Auto,
    Always,
    Never,
}

impl From<Color> for ColorMode {
    fn from(color: Color) -> Self {
        match color {
            Color::Auto => Self::Auto,
            Color::Always => Self::Always,
            Color::Never => Self::Never,
        }
    }
}

pub(crate) fn validate_render_options(
    format: Format,
    verbose: bool,
    color: Color,
) -> Result<(), &'static str> {
    if verbose && format == Format::Json {
        return Err("--verbose cannot be used with --format json");
    }
    if color == Color::Always && format == Format::Json {
        return Err("--color always cannot be used with --format json");
    }
    Ok(())
}
