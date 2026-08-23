use super::super::external::{
    ExternalOrigin, ExternalPluginDeclaration, ExternalPolicy, ExternalRootDeclaration,
    ExternalSkillDeclaration,
};
use super::super::{Agent, ManifestError, PathRoot, Scope};
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    external: &ExternalPolicy,
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    validate_roots(&external.roots, agents)?;
    let plugins = validate_plugins(&external.plugins, agents)?;
    validate_skills(&external.skills, agents, &plugins)
}

fn validate_roots(
    roots: &[ExternalRootDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let mut declarations = HashSet::new();
    for (index, root) in roots.iter().enumerate() {
        let field = |name: &str| format!("external.roots[{index}].{name}");
        super::validate_target(&field, root.agent, root.scope, agents)?;
        if root.origin != ExternalOrigin::System {
            return Err(ManifestError::new(
                field("origin"),
                "external roots only support system skills",
            ));
        }
        super::validate_path(&field("location.path"), &root.location.path)?;
        let expected = match root.scope {
            Scope::User => PathRoot::Home,
            Scope::Project => PathRoot::Repository,
        };
        if root.location.root != expected {
            return Err(ManifestError::new(
                field("location.root"),
                "location root is incompatible with external scope",
            ));
        }
        if !declarations.insert((root.agent, root.scope, root.origin, root.location.clone())) {
            return Err(ManifestError::new(
                field("location"),
                "duplicate external root",
            ));
        }
    }
    Ok(())
}

fn validate_plugins(
    plugins: &[ExternalPluginDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<HashSet<(Agent, Scope, String)>, ManifestError> {
    let mut declarations = HashSet::new();
    for (index, plugin) in plugins.iter().enumerate() {
        let field = |name: &str| format!("external.plugins[{index}].{name}");
        super::validate_target(&field, plugin.agent, plugin.scope, agents)?;
        super::skills::validate_slug(&field("id"), &plugin.id)?;
        if !declarations.insert((plugin.agent, plugin.scope, plugin.id.clone())) {
            return Err(ManifestError::new(field("id"), "duplicate external plugin"));
        }
    }
    Ok(declarations)
}

fn validate_skills(
    skills: &[ExternalSkillDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
    plugins: &HashSet<(Agent, Scope, String)>,
) -> Result<(), ManifestError> {
    let mut declarations = HashSet::new();
    for (index, skill) in skills.iter().enumerate() {
        let field = |name: &str| format!("external.skills[{index}].{name}");
        super::validate_target(&field, skill.agent, skill.scope, agents)?;
        super::skills::validate_slug(&field("slug"), &skill.slug)?;
        match (skill.origin, skill.plugin.as_deref()) {
            (ExternalOrigin::Managed, None) => {}
            (ExternalOrigin::Managed, Some(_)) => {
                return Err(ManifestError::new(
                    field("plugin"),
                    "managed external skills cannot name a plugin",
                ));
            }
            (ExternalOrigin::System, None) => {}
            (ExternalOrigin::System, Some(_)) => {
                return Err(ManifestError::new(
                    field("plugin"),
                    "system skills cannot name a plugin",
                ));
            }
            (ExternalOrigin::Plugin, None) => {
                return Err(ManifestError::new(
                    field("plugin"),
                    "plugin skills require a plugin identifier",
                ));
            }
            (ExternalOrigin::Plugin, Some(plugin))
                if !plugins.contains(&(skill.agent, skill.scope, plugin.to_owned())) =>
            {
                return Err(ManifestError::new(
                    field("plugin"),
                    "plugin skill requires a matching allowed plugin",
                ));
            }
            (ExternalOrigin::Plugin, Some(_)) => {}
        }
        if !declarations.insert((
            skill.agent,
            skill.scope,
            skill.origin,
            skill.plugin.clone(),
            skill.slug.clone(),
        )) {
            return Err(ManifestError::new(
                field("slug"),
                "duplicate external skill",
            ));
        }
    }
    Ok(())
}
