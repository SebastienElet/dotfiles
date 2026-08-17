use super::{Agent, RootedPath, Scope};
use serde::Deserialize;
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PromptDeclaration {
    pub(super) id: String,
    pub(super) source: RootedPath,
    pub(super) includes: Vec<PathBuf>,
    pub(super) variables: Vec<String>,
    pub(super) projections: Vec<PromptProjectionDeclaration>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PromptProjectionDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) representation: PromptRepresentation,
    pub(super) destination: RootedPath,
}

#[derive(Clone, Copy)]
pub struct Prompt<'a> {
    id: &'a str,
    source: &'a Path,
    includes: &'a [PathBuf],
    variables: &'a [String],
    projections: &'a [PromptProjectionDeclaration],
}

impl<'a> From<&'a PromptDeclaration> for Prompt<'a> {
    fn from(declaration: &'a PromptDeclaration) -> Self {
        Self {
            id: &declaration.id,
            source: &declaration.source.path,
            includes: &declaration.includes,
            variables: &declaration.variables,
            projections: &declaration.projections,
        }
    }
}

impl<'a> Prompt<'a> {
    pub fn id(self) -> &'a str {
        self.id
    }

    pub fn source(self) -> &'a Path {
        self.source
    }

    pub fn includes(self) -> impl ExactSizeIterator<Item = &'a Path> {
        self.includes.iter().map(PathBuf::as_path)
    }

    pub fn variables(self) -> impl ExactSizeIterator<Item = &'a str> {
        self.variables.iter().map(String::as_str)
    }

    pub fn projections(self) -> impl ExactSizeIterator<Item = PromptProjection<'a>> {
        self.projections.iter().map(PromptProjection::from)
    }
}

#[derive(Clone, Copy)]
pub struct PromptProjection<'a> {
    pub agent: Agent,
    pub scope: Scope,
    pub representation: PromptRepresentation,
    pub destination: &'a Path,
}

impl<'a> From<&'a PromptProjectionDeclaration> for PromptProjection<'a> {
    fn from(declaration: &'a PromptProjectionDeclaration) -> Self {
        Self {
            agent: declaration.agent,
            scope: declaration.scope,
            representation: declaration.representation,
            destination: &declaration.destination.path,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PromptRepresentation {
    File,
    Symlink,
    Rendered,
}

impl Display for PromptRepresentation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::File => "file",
            Self::Symlink => "symlink",
            Self::Rendered => "rendered",
        })
    }
}
