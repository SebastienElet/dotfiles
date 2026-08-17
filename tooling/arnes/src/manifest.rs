use clap::ValueEnum;
use serde::Deserialize;
use serde_yaml_ng::Value;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};

mod validation;

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
    resources: Vec<ResourceDeclaration>,
}

impl Manifest {
    pub fn combinations(&self) -> impl Iterator<Item = (Agent, Scope)> + '_ {
        self.agents
            .iter()
            .flat_map(|agent| agent.scopes.iter().map(move |scope| (agent.id, *scope)))
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

pub fn load(home: &Path) -> Result<Manifest, ManifestError> {
    let manifest = fs::read_to_string(home.join(MANIFEST_FILE)).map_err(|error| {
        let reason = match error.kind() {
            std::io::ErrorKind::NotFound => format!("{MANIFEST_FILE} was not found"),
            _ => format!("could not read {MANIFEST_FILE}"),
        };
        ManifestError::new("manifest", reason)
    })?;

    parse(&manifest)
}

pub fn parse(input: &str) -> Result<Manifest, ManifestError> {
    let value: Value = serde_yaml_ng::from_str(input)
        .map_err(|error| ManifestError::new("manifest", error.to_string()))?;
    validation::validate_value(&value)?;

    if let Some(field) = validation::secret_field(&value, "") {
        return Err(ManifestError::new(field, "secret values are not allowed"));
    }

    let deserializer = serde_yaml_ng::Deserializer::from_str(input);
    let manifest: Manifest = serde_path_to_error::deserialize(deserializer).map_err(|error| {
        let field = error.path().to_string();
        ManifestError::new(
            if field.is_empty() || field == "." {
                "manifest"
            } else {
                &field
            },
            error.into_inner().to_string(),
        )
    })?;
    validation::validate(&manifest)?;
    Ok(manifest)
}
