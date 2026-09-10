use super::*;

#[test]
fn installs_inputs_and_discards_fixture_home_with_its_lifetime()
-> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::prepare(
        &BTreeMap::from([("src/auth/session.ts".into(), "session".into())]),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )?;
    assert_eq!(
        std::fs::read_to_string(fixture.workspace.join("AGENTS.md"))?,
        "instructions"
    );
    assert_eq!(
        std::fs::read_to_string(
            fixture
                .workspace
                .join(".agents/skills/code-search/SKILL.md")
        )?,
        "skill"
    );
    assert_eq!(
        fixture.env.get("HOME").ok_or("missing fixture value")?,
        fixture.home.to_str().ok_or("missing fixture value")?
    );
    assert_eq!(
        fixture
            .env
            .get("CODEX_HOME")
            .ok_or("missing fixture value")?,
        fixture
            .home
            .join(".codex")
            .to_str()
            .ok_or("missing fixture value")?
    );
    assert!(!fixture.env.contains_key("CODEX_API_KEY"));
    assert!(!fixture.env.contains_key("PRIVATE_SENTINEL"));
    assert_eq!(std::fs::read_to_string(&fixture.observations)?, "");
    let root = fixture.root.clone();
    drop(fixture);
    assert!(!root.exists());
    Ok(())
}

#[test]
fn instruction_sources_override_fixture_entries() -> Result<(), Box<dyn std::error::Error>> {
    let fixture = Fixture::prepare(
        &BTreeMap::from([("AGENTS.md".into(), "fixture".into())]),
        "instructions",
        "skill",
        Path::new("/tmp/arnes"),
    )?;
    assert_eq!(
        std::fs::read_to_string(fixture.workspace.join("AGENTS.md"))?,
        "instructions"
    );
    Ok(())
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
