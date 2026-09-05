use super::support::*;

#[test]
fn fingerprint_tracks_project_instructions_config_hooks_and_skills() {
    let harness = Harness::new();
    fs::write(harness.repository.join("AGENTS.md"), "first instructions").unwrap();
    fs::create_dir(harness.repository.join(".codex")).unwrap();
    fs::write(
        harness.repository.join(".codex/config.toml"),
        "project='first'",
    )
    .unwrap();
    fs::write(
        harness.repository.join(".codex/hooks.json"),
        r#"{"hooks":{}}"#,
    )
    .unwrap();
    fs::create_dir_all(harness.repository.join(".codex/skills/example")).unwrap();
    fs::write(
        harness.repository.join(".codex/skills/example/SKILL.md"),
        "first skill",
    )
    .unwrap();
    let first = capture_run(&harness, "codex", "session_id", "one");

    fs::write(harness.repository.join("AGENTS.md"), "second instructions").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");

    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
    assert!(!first.to_string().contains("first instructions"));
}

#[path = "fingerprint/claude.rs"]
mod claude;
#[path = "fingerprint/codex.rs"]
mod codex;
#[path = "fingerprint/cursor_inventory.rs"]
mod cursor_inventory;
