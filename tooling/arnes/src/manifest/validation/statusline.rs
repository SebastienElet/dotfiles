use super::super::statusline::StatuslineDeclaration;
use super::super::{Agent, ManifestError, Scope};
use std::collections::{HashMap, HashSet};

pub(super) fn validate(
    declarations: &[StatuslineDeclaration],
    agents: &HashMap<Agent, HashSet<Scope>>,
) -> Result<(), ManifestError> {
    let mut projections = HashMap::new();
    for (index, declaration) in declarations.iter().enumerate() {
        let field = |name: &str| format!("statuslines[{index}].{name}");
        super::validate_target(&field, declaration.agent, declaration.scope, agents)?;
        if declaration.agent != Agent::Codex {
            return Err(ManifestError::new(
                field("agent"),
                "only codex status lines are supported",
            ));
        }
        if declaration.items.is_empty() {
            return Err(ManifestError::new(field("items"), "cannot be empty"));
        }
        for (item_index, item) in declaration.items.iter().enumerate() {
            if item.trim().is_empty() {
                return Err(ManifestError::new(
                    format!("statuslines[{index}].items[{item_index}]"),
                    "cannot be blank",
                ));
            }
        }
        if let Some(previous) = projections.insert((declaration.agent, declaration.scope), index) {
            return Err(ManifestError::new(
                field("scope"),
                format!("duplicates statuslines[{previous}] projection"),
            ));
        }
    }
    Ok(())
}
