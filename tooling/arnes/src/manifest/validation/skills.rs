use super::super::{
    Agent, ManifestError, ResourceDeclaration, ResourceKind, Scope, SkillDeclaration,
    SkillInstallation, SkillLayout,
};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

pub fn validate(
    skills: &[SkillDeclaration],
    resources: &[ResourceDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let mut identifiers = HashSet::new();
    for (skill_index, skill) in skills.iter().enumerate() {
        let field = |name: &str| format!("skills[{skill_index}].{name}");
        validate_slug(&field("slug"), &skill.slug)?;
        if skill.installations.is_empty() {
            return Err(ManifestError::new(
                field("installations"),
                "at least one leaf installation is required",
            ));
        }
        if !identifiers.insert(&skill.slug) {
            return Err(ManifestError::new(
                field("slug"),
                "duplicate skill identifier",
            ));
        }
        let mut installations = HashSet::new();
        for (index, installation) in skill.installations.iter().copied().enumerate() {
            let field = format!("skills[{skill_index}].installations[{index}]");
            validate_installation(&field, installation, agents, resources)?;
            if !installations.insert(installation) {
                return Err(ManifestError::new(field, "duplicate skill installation"));
            }
        }
    }
    Ok(())
}

fn validate_installation(
    field: &str,
    installation: SkillInstallation,
    agents: &HashMap<Agent, HashSet<Scope>>,
    resources: &[ResourceDeclaration],
) -> Result<(), ManifestError> {
    let Some(scopes) = agents.get(&installation.agent) else {
        return Err(ManifestError::new(
            format!("{field}.agent"),
            "agent is not declared",
        ));
    };
    if !scopes.contains(&installation.scope) {
        return Err(ManifestError::new(
            format!("{field}.scope"),
            "scope is not declared for this agent",
        ));
    }
    let projected = resources.iter().any(|resource| {
        resource.kind == ResourceKind::Skills
            && resource.agent == installation.agent
            && resource.scope == installation.scope
            && resource.layout == Some(SkillLayout::Leaves)
    });
    if projected {
        Ok(())
    } else {
        Err(ManifestError::new(field, "has no leaves skill projection"))
    }
}

fn validate_slug(field: &str, slug: &str) -> Result<(), ManifestError> {
    let path = Path::new(slug);
    let mut components = path.components();
    let valid = match (components.next(), components.next()) {
        (Some(Component::Normal(component)), None) => path.as_os_str() == component,
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(ManifestError::new(
            field,
            "must be one relative path component",
        ))
    }
}
