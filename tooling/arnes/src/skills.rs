use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Scope, SkillResource};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod discovery;
mod paths;
mod projection;
mod references;

struct Specification {
    root: &'static str,
    layout: Layout,
}

#[derive(Clone, Copy)]
enum Layout {
    Leaves,
    Root,
}

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
        .flat_map(|(agent, scope)| diagnose_one(roots, manifest, agent, scope))
        .collect()
}

fn diagnose_one(roots: &Roots, manifest: &Manifest, agent: Agent, scope: Scope) -> Vec<Diagnostic> {
    let specification = specification(agent, scope);
    let resources = manifest
        .skill_resources()
        .filter(|resource| resource.agent == agent && resource.scope == scope)
        .collect::<Vec<_>>();
    if resources.is_empty() {
        return vec![unsupported(Some(agent), Some(scope))];
    }
    let (supported, unsupported_resources): (Vec<_>, Vec<_>) = resources
        .into_iter()
        .partition(|resource| declaration_supported(resource, &specification));
    let mut diagnostics = unsupported_resources
        .into_iter()
        .map(|resource| unsupported_resource(&resource))
        .collect::<Vec<_>>();
    match specification.layout {
        Layout::Leaves => diagnostics.extend(diagnose_leaves(
            roots,
            agent,
            scope,
            &specification,
            supported,
        )),
        Layout::Root => diagnostics.extend(
            supported
                .iter()
                .flat_map(|resource| projection::root(roots, resource)),
        ),
    }
    diagnostics
}

fn diagnose_leaves(
    roots: &Roots,
    agent: Agent,
    scope: Scope,
    specification: &Specification,
    resources: Vec<SkillResource<'_>>,
) -> Vec<Diagnostic> {
    let declared = resources
        .iter()
        .map(|resource| resource.destination.to_owned())
        .collect::<HashSet<PathBuf>>();
    let mut diagnostics = resources
        .iter()
        .map(|resource| projection::leaf(roots, resource))
        .collect::<Vec<_>>();
    diagnostics.extend(discovery::unmanaged(
        roots,
        agent,
        scope,
        Path::new(specification.root),
        &declared,
    ));
    diagnostics
}

fn declaration_supported(resource: &SkillResource<'_>, specification: &Specification) -> bool {
    let source_root = Path::new(".agents/skills");
    let destination_root = Path::new(specification.root);
    match specification.layout {
        Layout::Root => resource.source == source_root && resource.destination == destination_root,
        Layout::Leaves => {
            resource.source.parent() == Some(source_root)
                && resource.destination.parent() == Some(destination_root)
                && resource.source.file_name() == resource.destination.file_name()
        }
    }
}

fn specification(agent: Agent, scope: Scope) -> Specification {
    let root = match (agent, scope) {
        (Agent::Claude, Scope::User) => ".claude/skills",
        (Agent::Cursor, Scope::User) => ".cursor/skills",
        (Agent::Codex, Scope::User) => ".agents/skills",
        (Agent::Claude, Scope::Project) => ".claude/skills",
        (Agent::Cursor, Scope::Project) => ".cursor/skills",
        (Agent::Codex, Scope::Project) => ".codex/skills",
    };
    let layout = match scope {
        Scope::User => Layout::Leaves,
        Scope::Project => Layout::Root,
    };
    Specification { root, layout }
}

fn unsupported(agent: Option<Agent>, scope: Option<Scope>) -> Diagnostic {
    let subject = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} skill projection"),
        (Some(agent), None) => format!("{agent} skill projections"),
        (None, Some(scope)) => format!("{scope} skill scope"),
        (None, None) => "skill projections".to_owned(),
    };
    Diagnostic::new(
        "skills",
        State::Unsupported,
        format!("{subject} is not declared or supported"),
    )
}

fn unsupported_resource(resource: &SkillResource<'_>) -> Diagnostic {
    Diagnostic::new(
        "skills",
        State::Unsupported,
        format!(
            "{} {} skill projection {} from {} to {} is unsupported",
            resource.agent,
            resource.scope,
            resource.id,
            resource.source.display(),
            resource.destination.display()
        ),
    )
}
