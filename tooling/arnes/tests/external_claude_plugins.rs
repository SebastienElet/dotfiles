#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use serde_json::json;
use skill_support::{MANIFEST, configured_fixture, run};
use support::Fixture;

fn manifest(plugin: bool, skill: bool) -> String {
    let plugins = if plugin {
        "    - { agent: claude, scope: user, id: demo@marketplace }\n"
    } else {
        ""
    };
    let skills = if skill {
        "    - { agent: claude, scope: user, origin: plugin, plugin: demo@marketplace, slug: hello }\n"
    } else {
        ""
    };
    MANIFEST.replacen(
        "resources:",
        &format!("external:\n  roots: []\n  plugins:\n{plugins}  skills:\n{skills}resources:"),
        1,
    )
}

fn plugin(fixture: &Fixture, version: &str, slug: &str) -> String {
    let root = format!(".claude/plugins/cache/marketplace/demo/{version}");
    fixture.write_home(
        format!("{root}/.claude-plugin/plugin.json"),
        &format!("{{\"name\":\"demo\",\"version\":\"{version}\",\"skills\":\"./skills\"}}"),
    );
    fixture.write_home(
        format!("{root}/skills/{slug}/SKILL.md"),
        "[missing](references/missing.md)\n",
    );
    fixture.home().join(root).display().to_string()
}

fn registry(fixture: &Fixture, installations: serde_json::Value) {
    fixture.write_home(
        ".claude/plugins/installed_plugins.json",
        &serde_json::to_string(&json!({
            "version": 2,
            "plugins": {"demo@marketplace": installations}
        }))
        .unwrap(),
    );
}

fn installation(path: &str, version: &str) -> serde_json::Value {
    json!({
        "scope": "user",
        "installPath": path,
        "version": version,
        "installedAt": "2026-08-17T00:00:00Z"
    })
}

#[test]
fn active_allowlisted_plugin_and_skill_are_external_and_healthy() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true, true));
    let path = plugin(&fixture, "1.0.0", "hello");
    registry(&fixture, json!([installation(&path, "1.0.0")]));
    fixture.write_home(
        ".claude/settings.json",
        "{\"enabledPlugins\":{\"demo@marketplace\":true}}",
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("plugin demo@marketplace origin=plugin ownership=external"));
    assert!(stdout.contains("version=1.0.0 exposure=enabled topology=healthy policy=allowed"));
    assert!(stdout.contains("skill hello origin=plugin ownership=external"));
    assert!(stdout.contains("container=demo@marketplace version=1.0.0"));
    assert!(!stdout.contains("local resource"));
}

#[test]
fn active_unexpected_plugin_reports_plugin_and_skill_policy_drift() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(false, false));
    let path = plugin(&fixture, "1.0.0", "hello");
    registry(&fixture, json!([installation(&path, "1.0.0")]));
    fixture.write_home(
        ".claude/settings.json",
        "{\"enabledPlugins\":{\"demo@marketplace\":true}}",
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 1, "{stdout}");
    assert_eq!(
        stdout.matches("drift skills: external claude user").count(),
        2
    );
    assert!(stdout.contains("plugin demo@marketplace"));
    assert!(stdout.contains("skill hello"));
    assert!(stdout.contains("policy=unexpected"));
}

#[test]
fn plugin_allowlist_does_not_allow_new_skills_implicitly() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true, false));
    let path = plugin(&fixture, "1.0.0", "future-capability");
    registry(&fixture, json!([installation(&path, "1.0.0")]));
    fixture.write_home(
        ".claude/settings.json",
        "{\"enabledPlugins\":{\"demo@marketplace\":true}}",
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 1, "{stdout}");
    let plugin = stdout
        .lines()
        .find(|line| line.contains("plugin demo@marketplace"))
        .unwrap();
    let skill = stdout
        .lines()
        .find(|line| line.contains("skill future-capability"))
        .unwrap();
    assert!(plugin.starts_with("healthy skills:"));
    assert!(skill.starts_with("drift skills:"));
}

#[test]
fn disabled_plugin_is_visible_without_being_reported_as_active() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(false, false));
    let path = plugin(&fixture, "1.0.0", "hello");
    registry(&fixture, json!([installation(&path, "1.0.0")]));
    fixture.write_home(
        ".claude/settings.json",
        "{\"enabledPlugins\":{\"demo@marketplace\":false}}",
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("exposure=disabled"));
    assert!(stdout.contains("activation=disabled"));
    assert!(!stdout.contains("available-not-runtime-observed"));
}

#[test]
fn orphan_cache_is_ignored_and_duplicate_active_versions_are_errors() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true, true));
    let current = plugin(&fixture, "1.0.0", "hello");
    plugin(&fixture, "0.9.0", "orphaned");
    registry(&fixture, json!([installation(&current, "1.0.0")]));
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("orphaned"));
    assert!(!stdout.contains("0.9.0"));

    registry(
        &fixture,
        json!([
            installation(&current, "1.0.0"),
            installation(&plugin(&fixture, "2.0.0", "hello"), "2.0.0")
        ]),
    );
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("topology=broken"));
    assert!(stdout.contains("ambiguous installed versions 1.0.0,2.0.0"));
}

#[test]
fn registry_and_manifest_version_mismatch_is_topological_error() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true, true));
    let path = plugin(&fixture, "1.0.0", "hello");
    fixture.write_home(
        ".claude/plugins/cache/marketplace/demo/1.0.0/.claude-plugin/plugin.json",
        "{\"name\":\"demo\",\"version\":\"2.0.0\",\"skills\":\"./skills\"}",
    );
    registry(&fixture, json!([installation(&path, "1.0.0")]));
    fixture.write_home(
        ".claude/settings.json",
        "{\"enabledPlugins\":{\"demo@marketplace\":true}}",
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("topology=broken policy=allowed"));
    assert!(stdout.contains("registry and plugin manifest versions differ"));
}

#[test]
fn absent_allowed_plugin_and_skill_create_no_policy_drift() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true, true));

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("demo@marketplace"));
    assert!(!stdout.contains("skill hello"));
}
