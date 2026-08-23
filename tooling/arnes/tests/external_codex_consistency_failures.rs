#[path = "support/codex.rs"]
pub mod codex_support;
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use codex_support::{install, marketplace, plugin};
use serde_json::json;
use skill_support::{MANIFEST, configured_fixture, run};

fn fixture() -> support::Fixture {
    let fixture = configured_fixture();
    let manifest = MANIFEST.replacen(
        "resources:",
        "external:\n  roots: []\n  plugins:\n    - { agent: codex, scope: user, id: demo@marketplace }\n  skills: []\nresources:",
        1,
    );
    fixture.write_home(".arnes.yaml", &manifest);
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    );
    fixture
}

fn diagnose(fixture: &support::Fixture) -> (i32, String, String) {
    run(
        fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )
}

#[test]
fn changed_exposure_between_config_and_resolver_is_unsupported() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        ".codex/.tmp/plugins/plugins/demo/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"1.0.0"}"#,
    );
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", false, &path)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("UNSUPPORTED plugin · enabled · unknown · allowed"));
    assert!(stdout.contains("Codex resolver exposure does not match configuration"));
}

#[test]
fn resolver_marketplace_must_match_the_plugin_identifier() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        ".codex/.tmp/plugins/plugins/demo/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"1.0.0"}"#,
    );
    let selected = plugin("demo@marketplace", "evil", "revision", true, &path);
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [selected], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("UNSUPPORTED plugin · enabled · unknown · allowed"));
    assert!(stdout.contains("Codex resolver marketplace does not match its identifier"));
}

#[test]
fn manifest_name_must_match_the_selected_plugin() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/revision/.codex-plugin/plugin.json",
        r#"{"name":"evil","version":"1.0.0","skills":["skills"]}"#,
    );
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/revision/skills/evil/SKILL.md",
        "evil\n",
    );
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &path)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("plugin manifest name does not match Codex selection"));
    assert!(!stdout.contains("skill evil"));
}

#[test]
fn empty_artifact_identifier_is_explicitly_unsupported() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        ".codex/.tmp/plugins/plugins/demo/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"1.0.0"}"#,
    );
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "", true, &path)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("UNSUPPORTED plugin · enabled · unknown · allowed"));
    assert!(stdout.contains("Codex resolver artifact identifier is invalid"));
}

#[test]
fn artifact_identifier_cannot_escape_the_cache_layout() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "../outside", true, &path)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex resolver artifact identifier is invalid"));
}
