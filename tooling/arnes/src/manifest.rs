use clap::ValueEnum;
use serde::Deserialize;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

mod commands;
mod config;
mod external;
mod hooks;
mod mcp;
mod parsing;
mod prompts;
mod rules;
mod validation;

pub use commands::{Command, CommandBinding};
pub use config::UserConfig;
pub use external::{ExternalOrigin, ExternalRoot, ExternalSkill};
pub use hooks::HookKind;
pub use mcp::McpRegistration;
pub use parsing::{load, parse};
pub use prompts::{Prompt, PromptProjection, PromptRepresentation};
pub use rules::RuleResource;

const MANIFEST_FILE: &str = ".arnes.yaml";
const SCHEMA_VERSION: u64 = 1;

#[derive(Debug)]
pub struct ManifestError {
    field: String,
    reason: String,
}

impl ManifestError {
    fn new(field: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            reason: reason.into(),
        }
    }
}

impl Display for ManifestError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.field, self.reason)
    }
}

impl std::error::Error for ManifestError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    #[serde(rename = "version")]
    _version: u64,
    agents: Vec<AgentDeclaration>,
    #[serde(default)]
    skills: Vec<SkillDeclaration>,
    #[serde(default)]
    external: external::ExternalPolicy,
    #[serde(default)]
    prompts: Vec<prompts::PromptDeclaration>,
    #[serde(default)]
    commands: Vec<commands::CommandDeclaration>,
    #[serde(default)]
    hooks: Vec<hooks::HookDeclaration>,
    #[serde(default)]
    mcp: Vec<mcp::McpDeclaration>,
    resources: Vec<ResourceDeclaration>,
}

impl Manifest {
    pub fn combinations(&self) -> impl Iterator<Item = (Agent, Scope)> + '_ {
        self.agents
            .iter()
            .flat_map(|agent| agent.scopes.iter().map(move |scope| (agent.id, *scope)))
    }

    pub fn user_config(&self, agent: Agent) -> Option<&UserConfig> {
        self.agents
            .iter()
            .find(|declaration| declaration.id == agent)
            .and_then(|declaration| declaration.user_config.as_ref())
    }

    pub fn instruction_resources(&self) -> impl Iterator<Item = InstructionResource<'_>> {
        self.resources
            .iter()
            .filter(|resource| resource.kind == ResourceKind::Instructions)
            .map(|resource| InstructionResource {
                id: &resource.id,
                agent: resource.agent,
                scope: resource.scope,
                source: &resource.source.path,
                destination: &resource.destination.path,
            })
    }

    pub fn skill_projections(&self) -> impl Iterator<Item = SkillProjection<'_>> {
        self.resources
            .iter()
            .filter(|resource| resource.kind == ResourceKind::Skills)
            .map(|resource| SkillProjection {
                id: &resource.id,
                agent: resource.agent,
                scope: resource.scope,
                layout: resource.layout.expect("skill projections have a layout"),
                source: &resource.source.path,
                destination: &resource.destination.path,
            })
    }

    pub fn installed_skills(&self, agent: Agent, scope: Scope) -> impl Iterator<Item = &str> {
        self.skills
            .iter()
            .filter(move |skill| {
                skill
                    .installations
                    .contains(&SkillInstallation { agent, scope })
            })
            .map(|skill| skill.slug.as_str())
    }

    pub fn external_roots(&self) -> impl Iterator<Item = ExternalRoot<'_>> {
        self.external.roots()
    }

    pub fn external_plugins(&self, agent: Agent, scope: Scope) -> impl Iterator<Item = &str> {
        self.external.plugins(agent, scope)
    }

    pub fn external_skills(
        &self,
        agent: Agent,
        scope: Scope,
    ) -> impl Iterator<Item = ExternalSkill<'_>> {
        self.external.skills(agent, scope)
    }

    pub fn prompts(&self) -> impl Iterator<Item = Prompt<'_>> {
        self.prompts.iter().map(Prompt::from)
    }

    pub(crate) fn resource_destinations(&self) -> impl Iterator<Item = (Scope, &Path)> {
        self.resources
            .iter()
            .map(|resource| (resource.scope, resource.destination.path.as_path()))
    }
}

#[derive(Clone, Copy)]
pub struct InstructionResource<'a> {
    pub id: &'a str,
    pub agent: Agent,
    pub scope: Scope,
    pub source: &'a Path,
    pub destination: &'a Path,
}

#[derive(Clone, Copy)]
pub struct SkillProjection<'a> {
    pub id: &'a str,
    pub agent: Agent,
    pub scope: Scope,
    pub layout: SkillLayout,
    pub source: &'a Path,
    pub destination: &'a Path,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillDeclaration {
    slug: String,
    installations: Vec<SkillInstallation>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct SkillInstallation {
    agent: Agent,
    scope: Scope,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AgentDeclaration {
    id: Agent,
    scopes: Vec<Scope>,
    user_config: Option<UserConfig>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResourceDeclaration {
    id: String,
    kind: ResourceKind,
    agent: Agent,
    scope: Scope,
    layout: Option<SkillLayout>,
    source: RootedPath,
    destination: RootedPath,
}

#[derive(Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SkillLayout {
    Leaves,
    Root,
}

#[derive(Clone, Eq, Hash, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
struct RootedPath {
    root: PathRoot,
    path: PathBuf,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum PathRoot {
    Home,
    Repository,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Agent {
    Claude,
    Cursor,
    Codex,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize, ValueEnum)]
#[serde(rename_all = "lowercase")]
pub enum Scope {
    User,
    Project,
}

#[derive(Clone, Copy, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum ResourceKind {
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
