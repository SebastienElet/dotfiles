use super::super::hooks::HookDeclaration;
use super::super::{Agent, HookKind, ManifestError, Scope};
use super::validate_target;
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    hooks: &[HookDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let mut identifiers = HashSet::new();
    for (hook_index, hook) in hooks.iter().enumerate() {
        let field = |name: &str| format!("hooks[{hook_index}].{name}");
        if !identifiers.insert(hook.id) {
            return Err(ManifestError::new(field("id"), "duplicate hook identifier"));
        }
        if hook.installations.is_empty() {
            return Err(ManifestError::new(
                field("installations"),
                "at least one hook installation is required",
            ));
        }
        let mut installations = HashSet::new();
        for (installation_index, installation) in hook.installations.iter().enumerate() {
            let installation_field = |name: &str| {
                format!("hooks[{hook_index}].installations[{installation_index}].{name}")
            };
            validate_target(
                &installation_field,
                installation.agent,
                installation.scope,
                agents,
            )?;
            if installation.scope != Scope::User {
                return Err(ManifestError::new(
                    installation_field("scope"),
                    "hooks only support the user scope",
                ));
            }
            if hook.id == HookKind::Handoff && installation.agent == Agent::Cursor {
                return Err(ManifestError::new(
                    installation_field("agent"),
                    "Cursor does not support the handoff hook",
                ));
            }
            if !installations.insert(*installation) {
                return Err(ManifestError::new(
                    format!("hooks[{hook_index}].installations[{installation_index}]"),
                    "duplicate hook installation",
                ));
            }
        }
    }
    Ok(())
}
