#[path = "support/codex.rs"]
pub mod codex_support;
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use codex_support::{install, install_script, marketplace, plugin};
use serde_json::json;
use skill_support::{MANIFEST, configured_fixture, run};
use std::os::unix::fs::symlink;

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
fn missing_marketplace_state_is_explicitly_unsupported() {
    let fixture = fixture();
    let path = fixture.home().join("must-not-be-read/demo");
    install(
        &fixture,
        json!({"marketplaces": []}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &path)], "available": []}),
    );
    let (code, stdout, _) = diagnose(&fixture);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("UNSUPPORTED plugin · enabled · unknown · allowed"));
    assert!(stdout.contains("marketplace selection is missing"));
    assert!(!stdout.contains("plugin manifest is missing"));
}

#[test]
fn malformed_resolver_metadata_is_explicitly_unsupported() {
    let fixture = fixture();
    fixture.write_home(".codex-test-marketplaces.json", "not json");
    fixture.write_home(".codex-test-plugins.json", r#"{"installed":[]}"#);
    install_script(
        &fixture,
        "#!/bin/sh\nif [ \"$2\" = \"marketplace\" ]; then file=\"$HOME/.codex-test-marketplaces.json\"; else file=\"$HOME/.codex-test-plugins.json\"; fi\nwhile IFS= read -r line || [ -n \"$line\" ]; do printf '%s\\n' \"$line\"; done < \"$file\"\n",
    );
    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex marketplace resolver returned invalid JSON"));
    assert!(stdout.contains("UNSUPPORTED plugin · enabled · unknown · allowed"));
}

#[test]
fn missing_resolved_artifact_is_broken_without_panicking() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(".codex/.tmp/plugins/.keep", "");
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &path)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("demo@marketplace@revision"));
    assert!(stdout.contains("ERROR plugin · enabled · broken · allowed"));
    assert!(stdout.contains("resolved plugin artifact is missing"));
}

#[test]
fn source_marketplace_payload_is_not_mistaken_for_the_active_artifact() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        ".codex/.tmp/plugins/plugins/demo/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"9.9.9"}"#,
    );
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &path)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("demo@marketplace@revision"));
    assert!(stdout.contains("resolved plugin artifact is missing"));
    assert!(!stdout.contains("9.9.9"));
}

#[test]
fn resolved_path_escape_is_rejected_before_inspection() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        "outside/demo/.codex-plugin/plugin.json",
        r#"{"name":"sentinel","version":"9.9.9"}"#,
    );
    fixture.write_home(".codex/plugins/cache/marketplace/demo/.keep", "");
    symlink(
        fixture.home().join("outside/demo"),
        fixture
            .home()
            .join(".codex/plugins/cache/marketplace/demo/revision"),
    )
    .unwrap();
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &path)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolved plugin path escapes the Codex plugin cache"));
    assert!(!stdout.contains("9.9.9"));
    assert!(!stdout.contains("sentinel"));
}

#[test]
fn resolved_artifact_cannot_alias_another_plugin_identity() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let source = root.join("plugins/demo");
    fixture.write_home(
        ".codex/plugins/cache/evil/demo/revision/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"9.9.9"}"#,
    );
    fixture.write_home(".codex/plugins/cache/marketplace/demo/.keep", "");
    symlink(
        "../../evil/demo/revision",
        fixture
            .home()
            .join(".codex/plugins/cache/marketplace/demo/revision"),
    )
    .unwrap();
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &source)], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolved plugin path aliases another cache identity"));
    assert!(!stdout.contains("9.9.9"));
}

#[test]
fn ambiguous_active_selection_is_explicitly_unsupported() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    fixture.write_home(
        ".codex/.tmp/plugins/plugins/demo/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"1.0.0"}"#,
    );
    let selected = plugin("demo@marketplace", "marketplace", "revision", true, &path);
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [selected.clone(), selected], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex resolver returned duplicate plugin selection"));
    assert!(stdout.contains("UNSUPPORTED plugin · enabled · unknown · allowed"));
}

#[test]
fn malformed_plugin_metadata_is_explicitly_unsupported() {
    let fixture = fixture();
    install(
        &fixture,
        json!({"marketplaces": []}),
        json!({"installed": [{"pluginId": "demo@marketplace"}], "available": []}),
    );

    let (code, stdout, _) = diagnose(&fixture);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex plugin resolver returned invalid JSON"));
}

#[test]
fn nullable_artifact_version_is_a_plugin_local_failure() {
    let fixture = fixture();
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    let mut selected = plugin("demo@marketplace", "marketplace", "revision", true, &path);
    selected["version"] = serde_json::Value::Null;
    fixture.write_home(
        ".codex/plugins/cache/marketplace/other/stable/.codex-plugin/plugin.json",
        r#"{"name":"other","version":"1.0.0"}"#,
    );
    let sibling = plugin(
        "other@marketplace",
        "marketplace",
        "stable",
        false,
        &root.join("plugins/other"),
    );
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [selected, sibling], "available": []}),
    );

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "skills",
            "--agent",
            "codex",
            "--scope",
            "user",
            "--verbose",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("Codex resolver has no active artifact identifier"));
    assert!(stdout.contains("other@marketplace@1.0.0"));
    assert!(!stdout.contains("returned invalid JSON"));
}
