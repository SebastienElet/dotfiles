use arnes::manifest::{self, Agent, Scope};

fn input(statuslines: &str) -> String {
    input_with_codex_scopes("user, project", statuslines)
}

fn input_with_codex_scopes(scopes: &str, statuslines: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\n  - id: cursor\n    scopes: [user]\n  - id: codex\n    scopes: [{scopes}]\nstatuslines:{statuslines}\nresources: []\n"
    )
}

fn error(statuslines: &str) -> String {
    manifest::parse(&input(statuslines))
        .err()
        .unwrap()
        .to_string()
}

#[test]
fn parses_ordered_codex_statusline_projections() {
    let manifest = manifest::parse(&input(
        "\n  - { agent: codex, scope: user, items: [model-with-reasoning, current-dir] }\n  - { agent: codex, scope: project, items: [context-used] }",
    ))
    .unwrap();
    let projections = manifest.statuslines().collect::<Vec<_>>();

    assert_eq!(projections.len(), 2);
    assert_eq!(
        (projections[0].agent, projections[0].scope),
        (Agent::Codex, Scope::User)
    );
    assert_eq!(
        projections[0].items,
        ["model-with-reasoning", "current-dir"]
    );
    assert_eq!(
        (projections[1].agent, projections[1].scope),
        (Agent::Codex, Scope::Project)
    );
    assert_eq!(projections[1].items, ["context-used"]);
}

#[test]
fn absent_statusline_declarations_remain_compatible() {
    let input = "version: 1\nagents:\n  - id: codex\n    scopes: [user, project]\nresources: []\n";

    assert_eq!(manifest::parse(input).unwrap().statuslines().count(), 0);
}

#[test]
fn legacy_statusline_resources_are_rejected_in_favor_of_top_level_declarations() {
    let input = "version: 1\nagents:\n  - id: codex\n    scopes: [user]\nstatuslines:\n  - { agent: codex, scope: user, items: [model] }\nresources:\n  - id: legacy-statusline\n    kind: statusline\n    agent: codex\n    scope: user\n    source: { root: repository, path: config.toml }\n    destination: { root: home, path: .codex/config.toml }\n";

    assert_eq!(
        manifest::parse(input).err().unwrap().to_string(),
        "resources[0].kind: statuslines must use normalized top-level declarations"
    );
}

#[test]
fn rejects_unsupported_duplicate_and_empty_statuslines() {
    for (statuslines, expected) in [
        (
            "\n  - { agent: claude, scope: user, items: [model] }",
            "statuslines[0].agent: only codex status lines are supported",
        ),
        (
            "\n  - { agent: cursor, scope: user, items: [model] }",
            "statuslines[0].agent: only codex status lines are supported",
        ),
        (
            "\n  - { agent: codex, scope: user, items: [] }",
            "statuslines[0].items: cannot be empty",
        ),
        (
            "\n  - { agent: codex, scope: user, items: [''] }",
            "statuslines[0].items[0]: cannot be blank",
        ),
        (
            "\n  - { agent: codex, scope: user, items: [model] }\n  - { agent: codex, scope: user, items: [dir] }",
            "statuslines[1].scope: duplicates statuslines[0] projection",
        ),
    ] {
        assert_eq!(error(statuslines), expected);
    }
}

#[test]
fn statusline_target_must_be_declared() {
    assert_eq!(
        manifest::parse(&input_with_codex_scopes(
            "user",
            "\n  - { agent: codex, scope: project, items: [model] }",
        ))
        .err()
        .unwrap()
        .to_string(),
        "statuslines[0].scope: scope is not declared for this agent"
    );
}
