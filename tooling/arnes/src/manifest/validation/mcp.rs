use super::super::mcp::McpDeclaration;
use super::super::{Agent, ManifestError, Scope};
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    declarations: &[McpDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let mut projections = HashMap::new();
    for (index, declaration) in declarations.iter().enumerate() {
        let field = |name: &str| format!("mcp[{index}].{name}");
        validate_declaration(declaration, index)?;
        super::validate_target(&field, declaration.agent, declaration.scope, agents)?;
        let identity = (
            declaration.agent,
            declaration.scope,
            declaration.name.as_str(),
        );
        if let Some(previous) = projections.insert(identity, index) {
            return Err(ManifestError::new(
                field("name"),
                format!("duplicates mcp[{previous}] projection"),
            ));
        }
    }
    Ok(())
}

fn validate_declaration(declaration: &McpDeclaration, index: usize) -> Result<(), ManifestError> {
    let field = |name: &str| format!("mcp[{index}].{name}");
    if !valid_name(&declaration.name) {
        return Err(ManifestError::new(
            field("name"),
            "must be lowercase ASCII kebab-case",
        ));
    }
    if declaration.command.trim().is_empty() {
        return Err(ManifestError::new(
            field("command"),
            "command cannot be blank",
        ));
    }
    if declaration.agent == Agent::Cursor && declaration.enabled.is_some() {
        return Err(ManifestError::new(
            field("enabled"),
            "cursor does not represent enabled state",
        ));
    }
    validate_environment(declaration, index)
}

fn validate_environment(declaration: &McpDeclaration, index: usize) -> Result<(), ManifestError> {
    let mut names = HashSet::new();
    for (environment_index, name) in declaration.environment.iter().enumerate() {
        let field = format!("mcp[{index}].environment[{environment_index}]");
        if !valid_environment_name(name) {
            return Err(ManifestError::new(
                field,
                "must be a shell environment variable name",
            ));
        }
        if !names.insert(name) {
            return Err(ManifestError::new(field, "duplicate environment reference"));
        }
    }
    Ok(())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

fn valid_environment_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'A'..=b'Z' | b'a'..=b'z' | b'_'))
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
