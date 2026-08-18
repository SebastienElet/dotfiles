use crate::Roots;
use crate::diagnostic::{Diagnostic, HumanSection, State};
use crate::manifest::{Agent, Manifest, Scope, SkillLayout, SkillProjection};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

mod discovery;
mod external;
mod projection;
mod references;

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
        let diagnostic = unsupported(agent, scope);
        return vec![match (agent, scope) {
            (Some(agent), Some(scope)) => diagnostic.with_human_section(section(agent, scope)),
            _ => diagnostic,
        }];
    }
    combinations
        .into_iter()
        .flat_map(|(agent, scope)| {
            diagnose_one(roots, manifest, agent, scope)
                .into_iter()
                .map(move |diagnostic| diagnostic.with_human_section(section(agent, scope)))
        })
        .collect()
}

fn section(agent: Agent, scope: Scope) -> HumanSection {
    HumanSection::new(format!("{agent}:{scope}"), agent.to_string().to_uppercase())
}

fn diagnose_one(roots: &Roots, manifest: &Manifest, agent: Agent, scope: Scope) -> Vec<Diagnostic> {
    let resources = manifest
        .skill_projections()
        .filter(|resource| resource.agent == agent && resource.scope == scope)
        .collect::<Vec<_>>();
    if resources.is_empty() {
        return std::iter::once(unsupported(Some(agent), Some(scope)))
            .chain(external::diagnose(roots, manifest, agent, scope))
            .collect();
    }
    let (supported, unsupported_resources): (Vec<_>, Vec<_>) =
        resources.into_iter().partition(declaration_supported);
    let mut diagnostics = unsupported_resources
        .into_iter()
        .map(|resource| unsupported_resource(&resource))
        .collect::<Vec<_>>();
    for resource in supported {
        match resource.layout {
            SkillLayout::Leaves => diagnostics.extend(diagnose_leaves(roots, manifest, &resource)),
            SkillLayout::Root => diagnostics.extend(diagnose_root(roots, manifest, &resource)),
        }
    }
    diagnostics.extend(external::diagnose(roots, manifest, agent, scope));
    diagnostics
}

fn diagnose_leaves(
    roots: &Roots,
    manifest: &Manifest,
    resource: &SkillProjection<'_>,
) -> Vec<Diagnostic> {
    let skills = manifest
        .installed_skills(resource.agent, resource.scope)
        .collect::<Vec<_>>();
    let declared = skills
        .iter()
        .map(|skill| resource.destination.join(*skill))
        .collect::<HashSet<PathBuf>>();
    let mut diagnostics = skills
        .iter()
        .map(|skill| projection::leaf(roots, resource, skill))
        .collect::<Vec<_>>();
    diagnostics.extend(discovery::unmanaged(
        roots,
        resource.agent,
        resource.scope,
        resource.destination,
        &declared,
        manifest,
    ));
    diagnostics
}

fn diagnose_root(
    roots: &Roots,
    manifest: &Manifest,
    resource: &SkillProjection<'_>,
) -> Vec<Diagnostic> {
    let skills = manifest.declared_skills().collect::<Vec<_>>();
    let declared = skills
        .iter()
        .map(|skill| resource.destination.join(*skill))
        .collect::<HashSet<PathBuf>>();
    let mut diagnostics = match projection::root(roots, resource, &skills) {
        Ok(diagnostics) => diagnostics,
        Err(diagnostic) => return vec![diagnostic],
    };
    diagnostics.extend(discovery::unmanaged(
        roots,
        resource.agent,
        resource.scope,
        resource.destination,
        &declared,
        manifest,
    ));
    diagnostics
}

fn declaration_supported(resource: &SkillProjection<'_>) -> bool {
    resource.source == Path::new(".agents/skills")
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

fn unsupported_resource(resource: &SkillProjection<'_>) -> Diagnostic {
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
