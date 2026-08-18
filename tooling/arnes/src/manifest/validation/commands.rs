use super::super::commands::CommandDeclaration;
use super::super::prompts::PromptDeclaration;
use super::super::{Agent, ManifestError, Scope};
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    commands: &[CommandDeclaration],
    prompts: &[PromptDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let prompt_ids = prompts
        .iter()
        .map(|prompt| prompt.id.as_str())
        .collect::<HashSet<_>>();
    let mut identities = HashMap::new();
    for (command_index, command) in commands.iter().enumerate() {
        validate_command(command, command_index, &prompt_ids)?;
        validate_bindings(command, command_index, agents, &mut identities)?;
    }
    Ok(())
}

fn validate_command(
    command: &CommandDeclaration,
    index: usize,
    prompt_ids: &HashSet<&str>,
) -> Result<(), ManifestError> {
    let field = |name: &str| format!("commands[{index}].{name}");
    if !valid_name(&command.name) {
        return Err(ManifestError::new(
            field("name"),
            "must be lowercase ASCII kebab-case",
        ));
    }
    if command.description.trim().is_empty() {
        return Err(ManifestError::new(
            field("description"),
            "description cannot be blank",
        ));
    }
    if !prompt_ids.contains(command.prompt.as_str()) {
        return Err(ManifestError::new(
            field("prompt"),
            "referenced prompt is not declared",
        ));
    }
    if command.bindings.is_empty() {
        return Err(ManifestError::new(
            field("bindings"),
            "at least one binding is required",
        ));
    }
    Ok(())
}

fn validate_bindings<'a>(
    command: &'a CommandDeclaration,
    command_index: usize,
    agents: &HashMap<Agent, HashSet<Scope>>,
    identities: &mut HashMap<(Agent, Scope, &'a str), (usize, usize)>,
) -> Result<(), ManifestError> {
    for (binding_index, binding) in command.bindings.iter().enumerate() {
        let field =
            |name: &str| format!("commands[{command_index}].bindings[{binding_index}].{name}");
        super::validate_target(&field, binding.agent, binding.scope, agents)?;
        let identity = (binding.agent, binding.scope, command.name.as_str());
        if let Some((previous_command, previous_binding)) =
            identities.insert(identity, (command_index, binding_index))
        {
            return Err(ManifestError::new(
                field("agent"),
                format!("duplicates commands[{previous_command}].bindings[{previous_binding}]"),
            ));
        }
    }
    Ok(())
}

fn valid_name(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}
