use super::super::{Agent, ManifestError, ResourceDeclaration, ResourceKind, Scope};
use std::path::Path;

pub fn validate(resource: &ResourceDeclaration, index: usize) -> Result<(), ManifestError> {
    if resource.kind != ResourceKind::Rules {
        return Ok(());
    }
    if resource.scope != Scope::User || !matches!(resource.agent, Agent::Claude | Agent::Cursor) {
        return Err(ManifestError::new(
            format!("resources[{index}].agent"),
            "rules only support claude or cursor user projections",
        ));
    }
    let valid_path = match resource.agent {
        Agent::Claude => is_rule_path(&resource.destination.path, ".claude/rules", "md"),
        Agent::Cursor => is_rule_path(&resource.destination.path, ".cursor/rules", "mdc"),
        Agent::Codex => false,
    };
    if !valid_path {
        let message = match resource.agent {
            Agent::Cursor => "cursor rules must be MDC files below .cursor/rules",
            Agent::Claude | Agent::Codex => {
                "claude rules must be Markdown files below .claude/rules"
            }
        };
        return Err(ManifestError::new(
            format!("resources[{index}].destination.path"),
            message,
        ));
    }
    Ok(())
}

fn is_rule_path(path: &Path, directory: &str, extension: &str) -> bool {
    path.starts_with(directory)
        && path != Path::new(directory)
        && path
            .extension()
            .is_some_and(|candidate| candidate == extension)
}
