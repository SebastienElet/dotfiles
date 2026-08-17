#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use serde_json::json;
use skill_support::{MANIFEST, configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;

fn policy(roots: &str, skills: &str) -> String {
    MANIFEST.replacen(
        "resources:",
        &format!("external:\n  roots:\n{roots}  plugins: []\n  skills:\n{skills}resources:"),
        1,
    )
}

#[test]
fn claude_registry_symlink_outside_temporary_home_is_not_read() {
    let fixture = support::Fixture::new();
    fixture.write_home(".arnes.yaml", &policy("", ""));
    fixture.write_home(".claude/plugins/.keep", "");
    let outside = tempfile::tempdir().unwrap();
    let outside_registry = outside.path().join("installed_plugins.json");
    fs::write(
        &outside_registry,
        r#"{"version":2,"plugins":{"real-home-sentinel@marketplace":[]}}"#,
    )
    .unwrap();
    symlink(
        &outside_registry,
        fixture
            .home()
            .join(".claude/plugins/installed_plugins.json"),
    )
    .unwrap();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("registry resolves outside the plugin root"));
    assert!(!stdout.contains("real-home-sentinel"));
    assert!(outside_registry.exists());
}

#[test]
fn claude_unknown_registry_scope_is_explicitly_unsupported() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &policy("", ""));
    fixture.write_home(
        ".claude/plugins/installed_plugins.json",
        &serde_json::to_string(&json!({
            "version": 2,
            "plugins": {"demo@marketplace": [{
                "scope": "future",
                "installPath": "/must/not/be/read",
                "version": "1.0.0"
            }]}
        }))
        .unwrap(),
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("exposure=unknown topology=unknown"));
    assert!(stdout.contains("unsupported installation scopes future"));
    assert!(!stdout.contains("must/not/be/read"));
}

#[test]
fn claude_install_path_outside_plugin_root_is_not_inspected() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &policy("", ""));
    fixture.write_home(
        ".claude/plugins/installed_plugins.json",
        r#"{"version":2,"plugins":{"demo@marketplace":[{"scope":"user","installPath":"/must/not/be/read","version":"1.0.0"}]}}"#,
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("installed plugin path escapes the plugin root"));
    assert!(!stdout.contains("plugin manifest is missing"));
}

#[test]
fn missing_roots_below_escaping_intermediate_links_are_errors() {
    for (agent, intermediate, root_path) in [
        ("codex", ".codex", ".codex/skills/.system"),
        ("cursor", ".cursor", ".cursor/system-skills"),
    ] {
        let fixture = support::Fixture::new();
        let roots = format!(
            "    - {{ agent: {agent}, scope: user, origin: system, location: {{ root: home, path: {root_path} }} }}\n"
        );
        fixture.write_home(".arnes.yaml", &policy(&roots, ""));
        let outside = tempfile::tempdir().unwrap();
        symlink(outside.path(), fixture.home().join(intermediate)).unwrap();

        let (code, stdout, _) = run(
            &fixture,
            &["doctor", "skills", "--agent", agent, "--scope", "user"],
        );

        assert_eq!(code, 2, "{stdout}");
        assert!(
            stdout.contains("root has an ancestor outside its scope"),
            "{stdout}"
        );
        if agent == "cursor" {
            assert!(
                stdout.contains("local plugin root has an ancestor outside HOME"),
                "{stdout}"
            );
        }
    }

    let fixture = support::Fixture::new();
    fixture.write_home(".arnes.yaml", &policy("", ""));
    let outside = tempfile::tempdir().unwrap();
    symlink(outside.path(), fixture.home().join(".claude")).unwrap();
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("plugin root is unreadable or resolves outside HOME"));
}

#[test]
fn duplicate_system_skill_slugs_across_roots_are_topological_errors() {
    let fixture = support::Fixture::new();
    let roots = "    - { agent: codex, scope: user, origin: system, location: { root: home, path: .codex/first } }\n    - { agent: codex, scope: user, origin: system, location: { root: home, path: .codex/second } }\n";
    let skills = "    - { agent: codex, scope: user, origin: system, slug: duplicate }\n";
    fixture.write_home(".arnes.yaml", &policy(roots, skills));
    fixture.write_home(".codex/first/duplicate/SKILL.md", "first\n");
    fixture.write_home(".codex/second/duplicate/SKILL.md", "second\n");

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 2, "{stdout}");
    assert_eq!(stdout.matches("skill duplicate origin=system").count(), 2);
    assert_eq!(
        stdout
            .matches("duplicate system skill slug across roots")
            .count(),
        2
    );
}
