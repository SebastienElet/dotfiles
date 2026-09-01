use super::{Agent, Manifest, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct McpDeclaration {
    pub(super) name: String,
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) command: String,
    #[serde(default)]
    pub(super) args: Vec<String>,
    #[serde(default)]
    pub(super) environment: Vec<String>,
    pub(super) enabled: Option<bool>,
}

#[derive(Clone, Copy)]
pub struct McpRegistration<'a> {
    pub name: &'a str,
    pub agent: Agent,
    pub scope: Scope,
    pub command: &'a str,
    pub args: &'a [String],
    pub environment: &'a [String],
    pub enabled: Option<bool>,
}

impl Manifest {
    pub fn mcp_registrations(&self) -> impl Iterator<Item = McpRegistration<'_>> {
        self.mcp.iter().map(|declaration| McpRegistration {
            name: &declaration.name,
            agent: declaration.agent,
            scope: declaration.scope,
            command: &declaration.command,
            args: &declaration.args,
            environment: &declaration.environment,
            enabled: declaration.enabled,
        })
    }
}
