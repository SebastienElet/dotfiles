use super::{Agent, Manifest, ResourceKind, Scope};
use std::path::Path;

impl Manifest {
    pub fn rule_resources(&self) -> impl Iterator<Item = RuleResource<'_>> {
        self.resources
            .iter()
            .filter(|resource| resource.kind == ResourceKind::Rules)
            .map(|resource| RuleResource {
                id: &resource.id,
                agent: resource.agent,
                scope: resource.scope,
                source: &resource.source.path,
                destination: &resource.destination.path,
            })
    }
}

#[derive(Clone, Copy)]
pub struct RuleResource<'a> {
    pub id: &'a str,
    pub agent: Agent,
    pub scope: Scope,
    pub source: &'a Path,
    pub destination: &'a Path,
}
