use super::*;

#[test]
fn installs_inputs_and_discards_fixture_home_with_its_lifetime() {
    let fixture = Fixture::prepare(
        &BTreeMap::from([("src/auth/session.ts".into(), "session".into())]),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.workspace.join("AGENTS.md")).unwrap(),
        "instructions"
    );
    assert_eq!(
        std::fs::read_to_string(
            fixture
                .workspace
                .join(".agents/skills/code-search/SKILL.md")
        )
        .unwrap(),
        "skill"
    );
    assert_eq!(
        fixture.env.get("HOME").unwrap(),
        fixture.home.to_str().unwrap()
    );
    assert_eq!(
        fixture.env.get("CODEX_HOME").unwrap(),
        fixture.home.join(".codex").to_str().unwrap()
    );
    assert!(!fixture.env.contains_key("CODEX_API_KEY"));
    assert!(!fixture.env.contains_key("PRIVATE_SENTINEL"));
    assert_eq!(std::fs::read_to_string(&fixture.observations).unwrap(), "");
    let root = fixture.root.clone();
    drop(fixture);
    assert!(!root.exists());
}

#[test]
fn instruction_sources_override_fixture_entries() {
    let fixture = Fixture::prepare(
        &BTreeMap::from([("AGENTS.md".into(), "fixture".into())]),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )
    .unwrap();
    assert_eq!(
        std::fs::read_to_string(fixture.workspace.join("AGENTS.md")).unwrap(),
        "instructions"
    );
}

#[test]
fn refuses_escaping_fixture_paths() {
    for path in ["../escape", "/escape"] {
        assert!(
            Fixture::prepare(
                &BTreeMap::from([(path.into(), String::new())]),
                "instructions",
                "skill",
                Path::new("/tmp/arnes")
            )
            .is_err()
        );
    }
}
