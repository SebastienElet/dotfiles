use super::super::prompts::PromptDeclaration;
use super::super::{Agent, ManifestError, PathRoot, ResourceDeclaration, Scope};
use std::collections::{HashMap, HashSet};
use std::path::{Component, Path, PathBuf};

pub(super) fn validate(
    prompts: &[PromptDeclaration],
    resources: &[ResourceDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let mut identifiers = HashMap::new();
    let mut sources = HashMap::new();
    let mut destinations = resources
        .iter()
        .enumerate()
        .map(|(index, resource)| (resource.destination.clone(), Destination::Resource(index)))
        .collect::<HashMap<_, _>>();

    for (prompt_index, prompt) in prompts.iter().enumerate() {
        let field = |name: &str| format!("prompts[{prompt_index}].{name}");
        if prompt.id.is_empty() {
            return Err(ManifestError::new(
                field("id"),
                "identifier cannot be empty",
            ));
        }
        if let Some(previous) = identifiers.insert(&prompt.id, prompt_index) {
            return Err(ManifestError::new(
                field("id"),
                format!("duplicates prompts[{previous}].id"),
            ));
        }
        super::validate_path(&field("source.path"), &prompt.source.path)?;
        if prompt.source.root != PathRoot::Repository {
            return Err(ManifestError::new(
                field("source.root"),
                "prompt sources must be repository-relative",
            ));
        }
        if let Some(previous) = sources.insert(prompt.source.clone(), prompt_index) {
            return Err(ManifestError::new(
                field("source"),
                format!("duplicates prompts[{previous}].source"),
            ));
        }
        validate_includes(prompt, prompt_index)?;
        validate_variables(prompt, prompt_index)?;
        validate_projections(prompt, prompt_index, agents, &mut destinations)?;
    }
    Ok(())
}

fn validate_includes(prompt: &PromptDeclaration, prompt_index: usize) -> Result<(), ManifestError> {
    let mut includes = HashMap::new();
    for (include_index, include) in prompt.includes.iter().enumerate() {
        let field = format!("prompts[{prompt_index}].includes[{include_index}]");
        let resolved = resolve_include(&prompt.source.path, include)
            .ok_or_else(|| ManifestError::new(&field, "path must stay within the repository"))?;
        if let Some(previous) = includes.insert(resolved, include_index) {
            return Err(ManifestError::new(
                field,
                format!("duplicates prompts[{prompt_index}].includes[{previous}]"),
            ));
        }
    }
    Ok(())
}

fn resolve_include(source: &Path, include: &Path) -> Option<PathBuf> {
    if include.as_os_str().is_empty() {
        return None;
    }
    let mut resolved = source.parent()?.to_path_buf();
    let mut has_file = false;
    for component in include.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => {
                resolved.push(part);
                has_file = true;
            }
            Component::ParentDir if resolved.pop() => has_file = false,
            _ => return None,
        }
    }
    has_file.then_some(resolved)
}

fn validate_variables(
    prompt: &PromptDeclaration,
    prompt_index: usize,
) -> Result<(), ManifestError> {
    let mut variables = HashMap::new();
    for (variable_index, variable) in prompt.variables.iter().enumerate() {
        let field = format!("prompts[{prompt_index}].variables[{variable_index}]");
        if !valid_variable(variable) {
            return Err(ManifestError::new(
                field,
                "must be an identifier without variable syntax",
            ));
        }
        if let Some(previous) = variables.insert(variable, variable_index) {
            return Err(ManifestError::new(
                field,
                format!("duplicates prompts[{prompt_index}].variables[{previous}]"),
            ));
        }
    }
    Ok(())
}

fn valid_variable(variable: &str) -> bool {
    let mut bytes = variable.bytes();
    match bytes.next() {
        Some(first) if first.is_ascii_alphabetic() || first == b'_' => {
            bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        }
        _ => false,
    }
}

fn validate_projections(
    prompt: &PromptDeclaration,
    prompt_index: usize,
    agents: &HashMap<Agent, HashSet<Scope>>,
    destinations: &mut HashMap<super::super::RootedPath, Destination>,
) -> Result<(), ManifestError> {
    let mut projections = HashMap::new();
    for (projection_index, projection) in prompt.projections.iter().enumerate() {
        let field =
            |name: &str| format!("prompts[{prompt_index}].projections[{projection_index}].{name}");
        super::validate_target(&field, projection.agent, projection.scope, agents)?;
        if let Some(previous) =
            projections.insert((projection.agent, projection.scope), projection_index)
        {
            return Err(ManifestError::new(
                field("agent"),
                format!("duplicates prompts[{prompt_index}].projections[{previous}]"),
            ));
        }
        super::validate_path(&field("destination.path"), &projection.destination.path)?;
        let expected_root = match projection.scope {
            Scope::User => PathRoot::Home,
            Scope::Project => PathRoot::Repository,
        };
        if projection.destination.root != expected_root {
            return Err(ManifestError::new(
                field("destination.root"),
                "destination root is incompatible with prompt projection scope",
            ));
        }
        validate_destination_registry(projection, &field)?;
        if prompt.source == projection.destination
            && !(projection.scope == Scope::Project
                && projection.representation == super::super::PromptRepresentation::File)
        {
            return Err(ManifestError::new(
                field("destination"),
                "source and destination must differ",
            ));
        }
        if let Some(previous) = destinations.insert(
            projection.destination.clone(),
            Destination::Prompt(prompt_index, projection_index),
        ) {
            return Err(ManifestError::new(field("destination"), previous.message()));
        }
    }
    Ok(())
}

fn validate_destination_registry(
    projection: &super::super::prompts::PromptProjectionDeclaration,
    field: &impl Fn(&str) -> String,
) -> Result<(), ManifestError> {
    let prefix = crate::prompts::capability::registry(projection.agent, projection.scope);
    if prefix.is_some_and(|prefix| {
        projection.destination.path == prefix
            || !projection.destination.path.starts_with(prefix)
            || projection
                .destination
                .path
                .extension()
                .and_then(|part| part.to_str())
                != Some("md")
    }) {
        Err(ManifestError::new(
            field("destination.path"),
            "destination must be a Markdown file inside the agent reusable-prompt registry",
        ))
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy)]
enum Destination {
    Resource(usize),
    Prompt(usize, usize),
}

impl Destination {
    fn message(self) -> String {
        match self {
            Self::Resource(index) => format!("duplicates resources[{index}].destination"),
            Self::Prompt(prompt, projection) => {
                format!("duplicates prompts[{prompt}].projections[{projection}].destination")
            }
        }
    }
}
