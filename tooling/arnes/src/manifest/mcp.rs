use super::{Agent, Manifest, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct McpDeclaration {
    pub(super) name: String,
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) command: Option<String>,
    pub(super) url: Option<String>,
    #[serde(default)]
    pub(super) args: Vec<String>,
    #[serde(default)]
    pub(super) environment: Vec<String>,
    pub(super) enabled: Option<bool>,
}

#[derive(Clone, Copy)]
pub enum McpTransport<'a> {
    Stdio {
        command: &'a str,
        args: &'a [String],
        environment: &'a [String],
    },
    Http {
        url: &'a str,
    },
}

#[derive(Clone, Copy)]
pub struct McpRegistration<'a> {
    pub name: &'a str,
    pub agent: Agent,
    pub scope: Scope,
    pub transport: McpTransport<'a>,
    pub enabled: Option<bool>,
}

impl Manifest {
    pub fn mcp_registrations(&self) -> impl Iterator<Item = McpRegistration<'_>> {
        self.0.mcp.iter().map(|declaration| McpRegistration {
            name: &declaration.name,
            agent: declaration.agent,
            scope: declaration.scope,
            transport: declaration.command.as_deref().map_or_else(
                || McpTransport::Http {
                    url: declaration.url.as_deref().unwrap_or_default(),
                },
                |command| McpTransport::Stdio {
                    command,
                    args: &declaration.args,
                    environment: &declaration.environment,
                },
            ),
            enabled: declaration.enabled,
        })
    }
}
