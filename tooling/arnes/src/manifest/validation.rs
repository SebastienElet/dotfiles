use super::{Agent, Manifest, ManifestError, PathRoot, ResourceDeclaration, SCHEMA_VERSION, Scope};
use serde_yaml_ng::Value;
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path};

mod skills;

pub(super) fn validate_value(value: &Value) -> Result<(), ManifestError> {
    let mapping = value
        .as_mapping()
        .ok_or_else(|| ManifestError::new("manifest", "expected a mapping"))?;
    let version = mapping
        .get(Value::String("version".to_owned()))
        .and_then(Value::as_u64)
        .ok_or_else(|| ManifestError::new("version", "required integer"))?;

    if version != SCHEMA_VERSION {
        return Err(ManifestError::new(
            "version",
            format!("unsupported version {version}; expected {SCHEMA_VERSION}"),
        ));
    }
    Ok(())
}

pub(super) fn secret_field(value: &Value, path: &str) -> Option<String> {
    match value {
        Value::Mapping(mapping) => mapping.iter().find_map(|(key, value)| {
            let key = key.as_str()?;
            let field = if path.is_empty() {
                key.to_owned()
            } else {
                format!("{path}.{key}")
            };
            if is_secret_name(key) {
                Some(field)
            } else {
                secret_field(value, &field)
            }
        }),
        Value::Sequence(sequence) => sequence
            .iter()
            .enumerate()
            .find_map(|(index, value)| secret_field(value, &format!("{path}[{index}]"))),
        _ => None,
    }
}

fn is_secret_name(name: &str) -> bool {
    name.to_ascii_lowercase()
        .replace('-', "_")
        .split('_')
        .any(|part| matches!(part, "secret" | "token" | "password" | "credential"))
}

pub(super) fn validate(manifest: &Manifest) -> Result<(), ManifestError> {
    let mut agents = HashMap::new();
    for (agent_index, agent) in manifest.agents.iter().enumerate() {
        if agents.contains_key(&agent.id) {
            return Err(ManifestError::new(
                format!("agents[{agent_index}].id"),
                "duplicate agent identifier",
            ));
        }
        let mut scopes = HashSet::new();
        for (scope_index, scope) in agent.scopes.iter().copied().enumerate() {
            if !scopes.insert(scope) {
                return Err(ManifestError::new(
                    format!("agents[{agent_index}].scopes[{scope_index}]"),
                    "duplicate scope identifier",
                ));
            }
        }
        agents.insert(agent.id, scopes);
    }

    validate_resources(&manifest.resources, &agents)?;
    skills::validate(&manifest.skills, &manifest.resources, &agents)
}

fn validate_resources(
    resources: &[ResourceDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let mut identifiers = HashMap::new();
    let mut destinations = HashMap::new();
    let mut skill_projections = HashMap::new();

    for (index, resource) in resources.iter().enumerate() {
        let field = |name: &str| format!("resources[{index}].{name}");
        if resource.id.is_empty() {
            return Err(ManifestError::new(
                field("id"),
                "identifier cannot be empty",
            ));
        }
        if let Some(previous) = identifiers.insert(&resource.id, index) {
            return Err(ManifestError::new(
                field("id"),
                format!("duplicates resources[{previous}].id"),
            ));
        }
        let Some(scopes) = agents.get(&resource.agent) else {
            return Err(ManifestError::new(field("agent"), "agent is not declared"));
        };
        if !scopes.contains(&resource.scope) {
            return Err(ManifestError::new(
                field("scope"),
                "scope is not declared for this agent",
            ));
        }
        validate_resource_paths(resource, index)?;
        validate_resource_layout(resource, index)?;
        if resource.kind == super::ResourceKind::Skills
            && let Some(previous) =
                skill_projections.insert((resource.agent, resource.scope), index)
        {
            return Err(ManifestError::new(
                field("agent"),
                format!("duplicates resources[{previous}] skill projection"),
            ));
        }
        if let Some(previous) = destinations.insert(&resource.destination, index) {
            return Err(ManifestError::new(
                field("destination"),
                format!("duplicates resources[{previous}].destination"),
            ));
        }
    }
    Ok(())
}

fn validate_resource_layout(
    resource: &ResourceDeclaration,
    index: usize,
) -> Result<(), ManifestError> {
    match (resource.kind, resource.layout) {
        (super::ResourceKind::Skills, None) => Err(ManifestError::new(
            format!("resources[{index}].layout"),
            "skill projection layout is required",
        )),
        (super::ResourceKind::Skills, Some(_)) | (_, None) => Ok(()),
        (_, Some(_)) => Err(ManifestError::new(
            format!("resources[{index}].layout"),
            "layout is only valid for skill projections",
        )),
    }
}

fn validate_resource_paths(
    resource: &ResourceDeclaration,
    index: usize,
) -> Result<(), ManifestError> {
    let source = format!("resources[{index}].source");
    let destination = format!("resources[{index}].destination");
    validate_path(&format!("{source}.path"), &resource.source.path)?;
    validate_path(&format!("{destination}.path"), &resource.destination.path)?;

    if resource.source.root != PathRoot::Repository {
        return Err(ManifestError::new(
            format!("{source}.root"),
            "resource sources must be repository-relative",
        ));
    }
    let expected_root = match resource.scope {
        Scope::User => PathRoot::Home,
        Scope::Project => PathRoot::Repository,
    };
    if resource.destination.root != expected_root {
        return Err(ManifestError::new(
            format!("{destination}.root"),
            "destination root is incompatible with resource scope",
        ));
    }
    if resource.source == resource.destination {
        return Err(ManifestError::new(
            destination,
            "source and destination must differ",
        ));
    }
    Ok(())
}

fn validate_path(field: &str, path: &Path) -> Result<(), ManifestError> {
    if path.as_os_str().is_empty() {
        return Err(ManifestError::new(field, "path cannot be empty"));
    }
    if path
        .components()
        .all(|part| matches!(part, Component::Normal(_)))
    {
        Ok(())
    } else {
        Err(ManifestError::new(
            field,
            "path must stay within its declared root",
        ))
    }
}
