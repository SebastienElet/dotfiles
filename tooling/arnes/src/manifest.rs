use clap::ValueEnum;
use serde::Deserialize;
use std::path::{Path, PathBuf};

mod commands;
mod config;
mod error;
mod external;
mod hooks;
mod mcp;
mod parsing;
mod prompts;
mod rules;
mod statusline;
mod validation;

pub use commands::{Command, CommandBinding};
pub use config::UserConfig;
pub use error::ManifestError;
pub use external::{ExternalOrigin, ExternalRoot, ExternalSkill};
pub use hooks::HookKind;
pub use mcp::McpRegistration;
pub use parsing::{load, parse};
pub use prompts::{Prompt, PromptProjection, PromptRepresentation};
pub use rules::RuleResource;
pub use statusline::Statusline;

const MANIFEST_FILE: &str = ".arnes.yaml";
const SCHEMA_VERSION: u64 = 1;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestData {
    #[serde(rename = "version")]
    version: u64,
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
    #[serde(default)]
    statuslines: Vec<statusline::StatuslineDeclaration>,
    resources: Vec<ResourceDeclaration>,
}

pub struct Manifest(ManifestData);

impl<'de> Deserialize<'de> for Manifest {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let manifest = Self(ManifestData::deserialize(deserializer)?);
        validation::validate(&manifest).map_err(serde::de::Error::custom)?;
        if manifest.0.version != SCHEMA_VERSION {
            return Err(serde::de::Error::custom("unsupported manifest version"));
        }
        Ok(manifest)
    }
}

impl Manifest {
    pub fn combinations(&self) -> impl Iterator<Item = (Agent, Scope)> + '_ {
        self.0
            .agents
            .iter()
            .flat_map(|agent| agent.scopes.iter().map(move |scope| (agent.id, *scope)))
    }

    #[must_use]
    pub fn user_config(&self, agent: Agent) -> Option<&UserConfig> {
        self.0
            .agents
            .iter()
            .find(|declaration| declaration.id == agent)
            .and_then(|declaration| declaration.user_config.as_ref())
    }

    pub fn instruction_resources(&self) -> impl Iterator<Item = InstructionResource<'_>> {
        self.0
            .resources
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
        self.0
            .resources
            .iter()
            .filter(|resource| resource.kind == ResourceKind::Skills)
            .filter_map(|resource| {
                Some(SkillProjection {
                    id: &resource.id,
                    agent: resource.agent,
                    scope: resource.scope,
                    layout: resource.layout?,
                    source: &resource.source.path,
                    destination: &resource.destination.path,
                })
            })
    }

    pub fn installed_skills(&self, agent: Agent, scope: Scope) -> impl Iterator<Item = &str> {
        self.0
            .skills
            .iter()
            .filter(move |skill| {
                skill
                    .installations
                    .contains(&SkillInstallation { agent, scope })
            })
            .map(|skill| skill.slug.as_str())
    }

    pub fn external_roots(&self) -> impl Iterator<Item = ExternalRoot<'_>> {
        self.0.external.roots()
    }

    pub fn external_plugins(&self, agent: Agent, scope: Scope) -> impl Iterator<Item = &str> {
        self.0.external.plugins(agent, scope)
    }

    pub fn external_skills(
        &self,
        agent: Agent,
        scope: Scope,
    ) -> impl Iterator<Item = ExternalSkill<'_>> {
        self.0.external.skills(agent, scope)
    }

    pub fn prompts(&self) -> impl Iterator<Item = Prompt<'_>> {
        self.0.prompts.iter().map(Prompt::from)
    }

    pub(crate) fn resource_destinations(&self) -> impl Iterator<Item = (Scope, &Path)> {
        self.0
            .resources
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
