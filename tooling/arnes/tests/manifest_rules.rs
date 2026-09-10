#![cfg(test)]
use arnes::manifest;
fn valid(agent: &str, destination: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let input = format!(
        "version: 1\nagents:\n  - id: {agent}\n    scopes: [user]\nresources:\n  - id: rule\n    kind: rules\n    agent: {agent}\n    scope: user\n    source: {{ root: repository, path: rule.mdc }}\n    destination: {{ root: home, path: {destination} }}\n"
    );
    manifest::parse(&input)?;
    Ok(())
}
fn error(
    agent: &str,
    scope: &str,
    destination: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let input = format!(
        "version: 1\nagents:\n  - id: {agent}\n    scopes: [{scope}]\nresources:\n  - id: rule\n    kind: rules\n    agent: {agent}\n    scope: {scope}\n    source: {{ root: repository, path: rule.md }}\n    destination: {{ root: home, path: {destination} }}\n"
    );
    match manifest::parse(&input) {
        Ok(_) => Err("rule manifest unexpectedly passed validation"
            .to_string()
            .into()),
        Err(error) => Ok(error.to_string()),
    }
}
#[test]
fn rules_reject_codex_user_projections() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_eq!(
        error("codex", "user", ".codex/rules/rule.md")?,
        "resources[0].agent: rules only support claude or cursor user projections"
    );
    Ok(())
}
#[test]
fn cursor_user_rules_use_the_mdc_discovery_directory()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    valid("cursor", ".cursor/rules/memory-governance-cursor.mdc")?;
    assert_eq!(
        error("cursor", "user", ".cursor/rules/rule.md")?,
        "resources[0].destination.path: cursor rules must be MDC files below .cursor/rules"
    );
    assert_eq!(
        error("cursor", "user", "nowhere/rule.mdc")?,
        "resources[0].destination.path: cursor rules must be MDC files below .cursor/rules"
    );
    Ok(())
}
#[test]
fn claude_rules_must_use_the_discovery_directory()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_eq!(
        error("claude", "user", "nowhere/rule.md")?,
        "resources[0].destination.path: claude rules must be Markdown files below .claude/rules"
    );
    assert_eq!(
        error("claude", "user", ".claude/rules/rule.txt")?,
        "resources[0].destination.path: claude rules must be Markdown files below .claude/rules"
    );
    Ok(())
}
