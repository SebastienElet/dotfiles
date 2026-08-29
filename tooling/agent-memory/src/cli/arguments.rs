use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "agent-memory")]
pub(super) struct Arguments {
    #[command(subcommand)]
    pub(super) command: Command,
}

#[derive(Debug, Subcommand)]
pub(super) enum Command {
    Admit(FormatArguments),
    Retrieve(RetrieveArguments),
    Confirm(ConfirmArguments),
    Audit(AuditArguments),
    Hook(HookArguments),
}

#[derive(Debug, Args)]
pub(super) struct FormatArguments {
    #[arg(long, value_enum)]
    pub(super) format: OutputFormat,
}

#[derive(Debug, Args)]
pub(super) struct RetrieveArguments {
    #[arg(long, required = true)]
    pub(super) query_stdin: bool,
    #[arg(long, value_enum)]
    pub(super) format: OutputFormat,
}

#[derive(Debug, Args)]
pub(super) struct ConfirmArguments {
    #[arg(long)]
    pub(super) id: String,
    #[arg(long, value_enum)]
    pub(super) status: HumanStatus,
    #[arg(long, required = true)]
    pub(super) reason_stdin: bool,
}

#[derive(Debug, Args)]
pub(super) struct AuditArguments {
    #[arg(long)]
    pub(super) include_terminal: bool,
    #[arg(long, value_enum)]
    pub(super) format: OutputFormat,
}

#[derive(Debug, Args)]
pub(super) struct HookArguments {
    #[arg(long, value_enum)]
    pub(super) agent: HookAgent,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum OutputFormat {
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum HumanStatus {
    Achieved,
    Abandoned,
    Superseded,
    Resolved,
    Confirmed,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub(super) enum HookAgent {
    Codex,
    Claude,
}
