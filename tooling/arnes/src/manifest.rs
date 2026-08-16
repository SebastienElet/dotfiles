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

pub struct Manifest {
    document: ManifestDocument,
    home: PathBuf,
    repository: PathBuf,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ManifestDocument {
    #[serde(rename = "version")]
    _version: u64,
    repository: RootedPath,
    agents: Vec<AgentDeclaration>,
    resources: Vec<ResourceDeclaration>,
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
    #[serde(rename = "kind")]
    _kind: ResourceKind,
    agent: Agent,
    scope: Scope,
    source: RootedPath,
    destination: RootedPath,
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

#[derive(Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Agent {
    Claude,
    Cursor,
    Codex,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
enum Scope {
    User,
    Project,
}

#[derive(Deserialize)]
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

    parse(&manifest, home)
}

pub fn parse(input: &str, home: &Path) -> Result<Manifest, ManifestError> {
    let value: Value = serde_yaml_ng::from_str(input)
        .map_err(|error| ManifestError::new("manifest", error.to_string()))?;
    validation::validate_value(&value)?;

    if let Some(field) = validation::secret_field(&value, "") {
        return Err(ManifestError::new(field, "secret values are not allowed"));
    }

    let deserializer = serde_yaml_ng::Deserializer::from_str(input);
    let document: ManifestDocument =
        serde_path_to_error::deserialize(deserializer).map_err(|error| {
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
    validation::validate(&document)?;
    let repository = home.join(&document.repository.path);
    Ok(Manifest {
        document,
        home: home.to_owned(),
        repository,
    })
}

impl Manifest {
    pub fn repository_root(&self) -> &Path {
        &self.repository
    }

    pub fn resource_paths(&self) -> impl Iterator<Item = (PathBuf, PathBuf)> + '_ {
        self.document.resources.iter().map(|resource| {
            (
                self.resolve(&resource.source),
                self.resolve(&resource.destination),
            )
        })
    }

    fn resolve(&self, path: &RootedPath) -> PathBuf {
        match path.root {
            PathRoot::Home => self.home.join(&path.path),
            PathRoot::Repository => self.repository.join(&path.path),
        }
    }
}
