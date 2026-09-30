use crate::Roots;
use crate::manifest::{Agent, Manifest, Scope};
use clap::ValueEnum;
use serde::Serialize;
use std::fmt::{self, Display};

mod config;
mod instructions;
mod links;
mod markdown;
mod mcp;
mod native_file;
mod selection;
mod sources;
mod statusline;

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncResource {
    Config,
    Instructions,
    Prompts,
    Commands,
    Mcp,
    Skills,
    Rules,
    Statusline,
}

impl Display for SyncResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Config => "config",
            Self::Instructions => "instructions",
            Self::Prompts => "prompts",
            Self::Commands => "commands",
            Self::Mcp => "mcp",
            Self::Skills => "skills",
            Self::Rules => "rules",
            Self::Statusline => "statusline",
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncState {
    Applied,
    Current,
    Refused,
    Failed,
    Empty,
    Unsupported,
}

#[derive(Debug, Serialize)]
pub struct SyncEntry {
    pub id: String,
    pub state: SyncState,
    pub message: String,
}

impl SyncEntry {
    fn new(id: impl Into<String>, state: SyncState, message: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            state,
            message: message.into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct SyncReport {
    pub resource: SyncResource,
    pub agent: String,
    pub scope: String,
    pub entries: Vec<SyncEntry>,
}

impl SyncReport {
    #[must_use]
    pub fn exit_code(&self) -> u8 {
        if self
            .entries
            .iter()
            .any(|entry| entry.state == SyncState::Failed)
        {
            return 2;
        }
        if self.entries.is_empty()
            || self
                .entries
                .iter()
                .any(|entry| !matches!(entry.state, SyncState::Applied | SyncState::Current))
        {
            return 1;
        }
        0
    }

    #[must_use]
    pub fn human(&self) -> String {
        let mut lines = vec![format!(
            "sync {} --agent {} --scope {}",
            self.resource, self.agent, self.scope
        )];
        for entry in &self.entries {
            lines.push(format!("{:?} {}: {}", entry.state, entry.id, entry.message));
        }
        lines.join("\n")
    }
}

#[must_use]
pub fn run(
    roots: &Roots,
    manifest: &Manifest,
    resource: SyncResource,
    agent: Agent,
    scope: Scope,
) -> SyncReport {
    let entries = match resource {
        SyncResource::Config => config::synchronize(roots, manifest, agent, scope),
        SyncResource::Instructions => instructions::synchronize(roots, manifest, agent, scope),
        SyncResource::Mcp => mcp::synchronize(roots, manifest, agent, scope),
        SyncResource::Prompts | SyncResource::Commands => {
            markdown::synchronize(roots, manifest, resource, agent, scope)
        }
        SyncResource::Skills | SyncResource::Rules => {
            match selection::select(roots, manifest, resource, agent, scope) {
                Err(entry) => vec![entry],
                Ok(selected) => synchronize_links(roots, manifest, scope, selected),
            }
        }
        SyncResource::Statusline => statusline::synchronize(roots, manifest, agent, scope),
    };
    SyncReport {
        resource,
        agent: agent.to_string(),
        scope: scope.to_string(),
        entries,
    }
}

fn synchronize_links(
    roots: &Roots,
    manifest: &Manifest,
    scope: Scope,
    selected: Vec<selection::LinkIntent>,
) -> Vec<SyncEntry> {
    let paths = selected
        .iter()
        .filter(|intent| {
            std::fs::symlink_metadata(intent.root.join(&intent.destination))
                .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
        })
        .map(|intent| intent.destination.clone())
        .collect::<Vec<_>>();
    if !paths.is_empty()
        && let Err(message) = sources::protect_mutations(roots, manifest, scope, &paths)
    {
        return vec![SyncEntry::new("selection", SyncState::Refused, message)];
    }
    links::synchronize(selected)
}
