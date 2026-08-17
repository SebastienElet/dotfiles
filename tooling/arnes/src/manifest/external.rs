use super::{Agent, RootedPath, Scope};
use serde::Deserialize;
use std::fmt::{self, Display};
use std::path::Path;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExternalPolicy {
    #[serde(default)]
    pub(super) roots: Vec<ExternalRootDeclaration>,
    #[serde(default)]
    pub(super) plugins: Vec<ExternalPluginDeclaration>,
    #[serde(default)]
    pub(super) skills: Vec<ExternalSkillDeclaration>,
}

impl ExternalPolicy {
    pub(super) fn roots(&self) -> impl Iterator<Item = ExternalRoot<'_>> {
        self.roots.iter().map(|root| ExternalRoot {
            agent: root.agent,
            scope: root.scope,
            origin: root.origin,
            path: &root.location.path,
        })
    }

    pub(super) fn plugins(&self, agent: Agent, scope: Scope) -> impl Iterator<Item = &str> {
        self.plugins
            .iter()
            .filter(move |plugin| plugin.agent == agent && plugin.scope == scope)
            .map(|plugin| plugin.id.as_str())
    }

    pub(super) fn skills(
        &self,
        agent: Agent,
        scope: Scope,
    ) -> impl Iterator<Item = ExternalSkill<'_>> {
        self.skills
            .iter()
            .filter(move |skill| skill.agent == agent && skill.scope == scope)
            .map(|skill| ExternalSkill {
                origin: skill.origin,
                plugin: skill.plugin.as_deref(),
                slug: &skill.slug,
            })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExternalRootDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) origin: ExternalOrigin,
    pub(super) location: RootedPath,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExternalPluginDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) id: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ExternalSkillDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) origin: ExternalOrigin,
    pub(super) plugin: Option<String>,
    pub(super) slug: String,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExternalOrigin {
    Managed,
    System,
    Plugin,
}

impl Display for ExternalOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Managed => "managed",
            Self::System => "system",
            Self::Plugin => "plugin",
        })
    }
}

#[derive(Clone, Copy)]
pub struct ExternalRoot<'a> {
    pub agent: Agent,
    pub scope: Scope,
    pub origin: ExternalOrigin,
    pub path: &'a Path,
}

#[derive(Clone, Copy)]
pub struct ExternalSkill<'a> {
    pub origin: ExternalOrigin,
    pub plugin: Option<&'a str>,
    pub slug: &'a str,
}
