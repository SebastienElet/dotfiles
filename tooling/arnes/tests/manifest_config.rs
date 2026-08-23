use arnes::manifest;

fn error(agent: &str) -> String {
    let input = format!("version: 1\nagents:\n{agent}resources: []\n");
    manifest::parse(&input).err().unwrap().to_string()
}

#[test]
fn user_defaults_require_user_scope() {
    assert_eq!(
        error("  - id: codex\n    scopes: [project]\n    user_config:\n      model: gpt-5.6-sol\n"),
        "agents[0].user_config: requires the user scope"
    );
}

#[test]
fn agent_capabilities_are_validated() {
    for (agent, expected) in [
        (
            "  - id: cursor\n    scopes: [user]\n    user_config:\n      model: auto\n      effort: high\n",
            "agents[0].user_config.effort: cursor does not expose a persistent effort setting",
        ),
        (
            "  - id: claude\n    scopes: [user]\n    user_config:\n      model: opus\n      context_window: 1000000\n",
            "agents[0].user_config.context_window: claude does not expose this persistent setting",
        ),
        (
            "  - id: codex\n    scopes: [user]\n    user_config:\n      model: gpt-5.6-sol\n      max_mode: true\n",
            "agents[0].user_config.max_mode: codex does not expose max mode",
        ),
    ] {
        assert_eq!(error(agent), expected);
    }
}

#[test]
fn model_and_windows_are_validated() {
    for (agent, expected) in [
        (
            "  - id: codex\n    scopes: [user]\n    user_config:\n      model: \"\"\n",
            "agents[0].user_config.model: cannot be empty",
        ),
        (
            "  - id: claude\n    scopes: [user]\n    user_config:\n      model: opus\n      auto_compact_window: 0\n",
            "agents[0].user_config.auto_compact_window: must be greater than zero",
        ),
        (
            "  - id: claude\n    scopes: [user]\n    user_config:\n      model: opus\n      auto_compact_window: 99999\n",
            "agents[0].user_config.auto_compact_window: must be between 100000 and 1000000",
        ),
        (
            "  - id: codex\n    scopes: [user]\n    user_config:\n      model: gpt-5.6-sol\n      context_window: 270000\n      auto_compact_window: 270000\n",
            "agents[0].user_config.auto_compact_window: must be smaller than context_window",
        ),
    ] {
        assert_eq!(error(agent), expected);
    }
}
