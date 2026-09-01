use arnes::manifest::{self, Agent, Scope};

fn input(mcp: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user, project]\n  - id: cursor\n    scopes: [project]\nmcp:{mcp}\nresources: []\n"
    )
}

fn error(mcp: &str) -> String {
    manifest::parse(&input(mcp)).err().unwrap().to_string()
}

#[test]
fn parses_one_explicit_mcp_projection() {
    let manifest = manifest::parse(&input(
        "\n  - name: apple-notes\n    agent: claude\n    scope: project\n    command: notes-mcp\n    args: [--stdio]\n    environment: [NOTES_PROFILE]\n    enabled: true",
    ))
    .unwrap();
    let registration = manifest.mcp_registrations().next().unwrap();

    assert_eq!(registration.name, "apple-notes");
    assert_eq!(
        (registration.agent, registration.scope),
        (Agent::Claude, Scope::Project)
    );
    assert_eq!(registration.command, "notes-mcp");
    assert_eq!(registration.args, ["--stdio"]);
    assert_eq!(registration.environment, ["NOTES_PROFILE"]);
    assert_eq!(registration.enabled, Some(true));
}

#[test]
fn absent_mcp_declarations_remain_compatible() {
    assert_eq!(
        manifest::parse(&input(" []"))
            .unwrap()
            .mcp_registrations()
            .count(),
        0
    );
}

#[test]
fn rejects_invalid_and_duplicate_declarations() {
    for (mcp, expected) in [
        (
            "\n  - { name: Bad_Name, agent: claude, scope: user, command: mcp }",
            "mcp[0].name: must be lowercase ASCII kebab-case",
        ),
        (
            "\n  - { name: managed, agent: claude, scope: user, command: '' }",
            "mcp[0].command: command cannot be blank",
        ),
        (
            "\n  - { name: managed, agent: claude, scope: user, command: ../mcp }",
            "mcp[0].command: relative command must stay within its scope root",
        ),
        (
            "\n  - { name: managed, agent: claude, scope: user, command: mcp, environment: [GOOD, BAD-NAME] }",
            "mcp[0].environment[1]: must be a shell environment variable name",
        ),
        (
            "\n  - { name: managed, agent: claude, scope: user, command: mcp, environment: [TOKEN, TOKEN] }",
            "mcp[0].environment[1]: duplicate environment reference",
        ),
        (
            "\n  - { name: managed, agent: claude, scope: user, command: one }\n  - { name: managed, agent: claude, scope: user, command: two }",
            "mcp[1].name: duplicates mcp[0] projection",
        ),
        (
            "\n  - { name: managed, agent: cursor, scope: project, command: mcp, enabled: true }",
            "mcp[0].enabled: cursor does not represent enabled state",
        ),
    ] {
        assert_eq!(error(mcp), expected);
    }
}

#[test]
fn targets_must_be_declared() {
    assert_eq!(
        error("\n  - { name: managed, agent: cursor, scope: user, command: mcp }"),
        "mcp[0].scope: scope is not declared for this agent"
    );
}
