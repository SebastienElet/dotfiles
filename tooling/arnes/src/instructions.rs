use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Scope};

mod checks;
mod includes;
mod projection;

pub fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    let combinations = manifest
        .combinations()
        .filter(|(candidate, _)| agent.is_none_or(|agent| agent == *candidate))
        .filter(|(_, candidate)| scope.is_none_or(|scope| scope == *candidate))
        .collect::<Vec<_>>();

    if combinations.is_empty() {
        return vec![unsupported(agent, scope)];
    }

    combinations
        .into_iter()
        .flat_map(|(agent, scope)| {
            let resources = manifest
                .instruction_resources()
                .filter(|resource| resource.agent == agent && resource.scope == scope)
                .collect::<Vec<_>>();
            let Some(kind) = projection::kind(agent, scope) else {
                return vec![unsupported(Some(agent), Some(scope))];
            };
            if resources.is_empty() {
                return vec![unsupported(Some(agent), Some(scope))];
            }
            resources
                .iter()
                .map(|resource| projection::diagnose(roots, resource, &resources, kind))
                .collect()
        })
        .collect()
}

fn unsupported(agent: Option<Agent>, scope: Option<Scope>) -> Diagnostic {
    let subject = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} instruction projection"),
        (Some(agent), None) => format!("{agent} instruction projection"),
        (None, Some(scope)) => format!("{scope} instruction scope"),
        (None, None) => "instruction projections".to_owned(),
    };
    Diagnostic::new(
        "instructions",
        State::Unsupported,
        format!("{subject} is not declared or supported"),
    )
}
