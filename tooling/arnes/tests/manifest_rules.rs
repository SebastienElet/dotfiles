use arnes::manifest;

fn valid(agent: &str, destination: &str) {
    let input = format!(
        "version: 1\nagents:\n  - id: {agent}\n    scopes: [user]\nresources:\n  - id: rule\n    kind: rules\n    agent: {agent}\n    scope: user\n    source: {{ root: repository, path: rule.mdc }}\n    destination: {{ root: home, path: {destination} }}\n"
    );
    manifest::parse(&input).unwrap();
}

fn error(agent: &str, scope: &str, destination: &str) -> String {
    let input = format!(
        "version: 1\nagents:\n  - id: {agent}\n    scopes: [{scope}]\nresources:\n  - id: rule\n    kind: rules\n    agent: {agent}\n    scope: {scope}\n    source: {{ root: repository, path: rule.md }}\n    destination: {{ root: home, path: {destination} }}\n"
    );
    match manifest::parse(&input) {
        Ok(_) => panic!("rule manifest unexpectedly passed validation"),
        Err(error) => error.to_string(),
    }
}

#[test]
fn rules_reject_codex_user_projections() {
    assert_eq!(
        error("codex", "user", ".codex/rules/rule.md"),
        "resources[0].agent: rules only support claude or cursor user projections"
    );
}

#[test]
fn cursor_user_rules_use_the_mdc_discovery_directory() {
    valid("cursor", ".cursor/rules/memory-governance-cursor.mdc");
    assert_eq!(
        error("cursor", "user", ".cursor/rules/rule.md"),
        "resources[0].destination.path: cursor rules must be MDC files below .cursor/rules"
    );
    assert_eq!(
        error("cursor", "user", "nowhere/rule.mdc"),
        "resources[0].destination.path: cursor rules must be MDC files below .cursor/rules"
    );
}

#[test]
fn claude_rules_must_use_the_discovery_directory() {
    assert_eq!(
        error("claude", "user", "nowhere/rule.md"),
        "resources[0].destination.path: claude rules must be Markdown files below .claude/rules"
    );
    assert_eq!(
        error("claude", "user", ".claude/rules/rule.txt"),
        "resources[0].destination.path: claude rules must be Markdown files below .claude/rules"
    );
}
