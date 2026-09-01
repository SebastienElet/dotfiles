use super::{Agent, Manifest, Scope};
use serde::Deserialize;
use std::fmt::{self, Display};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HookKind {
    Measurement,
    Handoff,
    Memory,
}

impl Display for HookKind {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Measurement => "measurement",
            Self::Handoff => "handoff",
            Self::Memory => "memory",
        })
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HookDeclaration {
    pub(super) id: HookKind,
    pub(super) installations: Vec<HookInstallation>,
}

#[derive(Clone, Copy, Eq, Hash, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct HookInstallation {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
}

pub(super) fn installed(
    declarations: &[HookDeclaration],
    agent: Agent,
    scope: Scope,
) -> impl Iterator<Item = HookKind> + '_ {
    declarations
        .iter()
        .filter(move |declaration| {
            declaration
                .installations
                .contains(&HookInstallation { agent, scope })
        })
        .map(|declaration| declaration.id)
}

impl Manifest {
    pub fn hooks(&self, agent: Agent, scope: Scope) -> impl Iterator<Item = HookKind> + '_ {
        installed(&self.hooks, agent, scope)
    }
}
