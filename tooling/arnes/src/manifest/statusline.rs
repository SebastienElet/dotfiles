use super::{Agent, Manifest, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct StatuslineDeclaration {
    pub(super) agent: Agent,
    pub(super) scope: Scope,
    pub(super) items: Vec<String>,
}

#[derive(Clone, Copy)]
pub struct Statusline<'a> {
    pub agent: Agent,
    pub scope: Scope,
    pub items: &'a [String],
}

impl Manifest {
    pub fn statuslines(&self) -> impl Iterator<Item = Statusline<'_>> {
        self.statuslines.iter().map(|declaration| Statusline {
            agent: declaration.agent,
            scope: declaration.scope,
            items: &declaration.items,
        })
    }
}
