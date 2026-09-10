use super::support::*;
#[test]
fn fingerprint_tracks_project_instructions_config_hooks_and_skills()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    fs::write(harness.repository.join("AGENTS.md"), "first instructions")?;
    fs::create_dir(harness.repository.join(".codex"))?;
    fs::write(
        harness.repository.join(".codex/config.toml"),
        "project='first'",
    )?;
    fs::write(
        harness.repository.join(".codex/hooks.json"),
        r#"{"hooks":{}}"#,
    )?;
    fs::create_dir_all(harness.repository.join(".codex/skills/example"))?;
    fs::write(
        harness.repository.join(".codex/skills/example/SKILL.md"),
        "first skill",
    )?;
    let first = capture_run(&harness, "codex", "session_id", "one")?;
    fs::write(harness.repository.join("AGENTS.md"), "second instructions")?;
    let second = capture_run(&harness, "codex", "session_id", "two")?;
    assert_ne!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    assert!(!first.to_string().contains("first instructions"));
    Ok(())
}
#[path = "fingerprint/claude.rs"]
mod claude;
#[path = "fingerprint/codex.rs"]
mod codex;
#[path = "fingerprint/cursor_inventory.rs"]
mod cursor_inventory;
