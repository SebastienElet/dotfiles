use arnes::manifest;

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
fn rules_only_support_claude_user_projections() {
    assert_eq!(
        error("codex", "user", ".codex/rules/rule.md"),
        "resources[0].agent: rules only support claude user projections"
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
