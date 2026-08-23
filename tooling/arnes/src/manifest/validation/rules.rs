use super::super::{Agent, ManifestError, ResourceDeclaration, ResourceKind, Scope};
use std::path::Path;

pub fn validate(resource: &ResourceDeclaration, index: usize) -> Result<(), ManifestError> {
    if resource.kind != ResourceKind::Rules {
        return Ok(());
    }
    if resource.agent != Agent::Claude || resource.scope != Scope::User {
        return Err(ManifestError::new(
            format!("resources[{index}].agent"),
            "rules only support claude user projections",
        ));
    }
    if !is_claude_rule_path(&resource.destination.path) {
        return Err(ManifestError::new(
            format!("resources[{index}].destination.path"),
            "claude rules must be Markdown files below .claude/rules",
        ));
    }
    Ok(())
}

fn is_claude_rule_path(path: &Path) -> bool {
    path.starts_with(".claude/rules")
        && path != Path::new(".claude/rules")
        && path.extension().is_some_and(|extension| extension == "md")
}
