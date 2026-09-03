#[path = "support/codex.rs"]
pub mod codex_support;
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use codex_support::{install, marketplace, plugin};
use serde_json::json;
use skill_support::{MANIFEST, configured_fixture, run};

fn manifest(plugin_allowed: bool, skill_allowed: bool) -> String {
    let plugins = if plugin_allowed {
        "    - { agent: codex, scope: user, id: demo@marketplace }\n"
    } else {
        ""
    };
    let skills = if skill_allowed {
        "    - { agent: codex, scope: user, origin: plugin, plugin: demo@marketplace, slug: brainstorm }\n"
    } else {
        ""
    };
    MANIFEST.replacen(
        "resources:",
        &format!(
            "external:\n  roots:\n    - {{ agent: codex, scope: user, origin: system, location: {{ root: home, path: .codex/skills/.system }} }}\n  plugins:\n{plugins}  skills:\n{skills}resources:"
        ),
        1,
    )
}

#[test]
fn active_plugin_uses_codex_selection_and_inventories_skills() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true, true));
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n[[skills.config]]\nname = 'review'\nenabled = false\n",
    );
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/11c74d6b/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"5.1.3","skills":["skills"]}"#,
    );
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/11c74d6b/skills/brainstorm/SKILL.md",
        "brainstorm\n",
    );
    fixture.write_home(
        ".codex/.tmp/plugins/plugins/demo/skills/source-only/SKILL.md",
        "source only\n",
    );
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "11c74d6b", true, &path)], "available": []}),
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
    assert!(stdout.contains("codex user plugin demo@marketplace@5.1.3"));
    assert!(stdout.contains("artifact=11c74d6b"));
    assert!(stdout.contains("plugin · enabled · healthy · allowed"));
    assert!(stdout.contains("skill brainstorm · enabled · healthy · allowed"));
    assert!(!stdout.contains("@?"));
    assert!(!stdout.contains("path=unknown"));
    assert!(!stdout.contains("topology=unknown"));
    assert!(!stdout.contains("source-only"));
}

#[test]
fn disabled_plugin_is_classified_from_the_resolved_artifact() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(false, false));
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = false\n",
    );
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/revision/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"1.0.0"}"#,
    );
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", false, &path)], "available": []}),
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
    assert!(stdout.contains("demo@marketplace@1.0.0"));
    assert!(stdout.contains("plugin · disabled · healthy · unexpected"));
    assert!(!stdout.contains("UNSUPPORTED plugin"));
}

#[test]
fn a_lone_cache_artifact_never_proves_activation() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(false, false));
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    );
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/0.9.0/skills/orphan/SKILL.md",
        "orphan\n",
    );
    install(
        &fixture,
        json!({"marketplaces": []}),
        json!({"installed": [], "available": []}),
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("DRIFT plugin · enabled · unknown · unexpected"));
    assert!(stdout.contains("UNSUPPORTED external codex user plugin resolution"));
    assert!(stdout.contains("Codex resolver did not select this configured plugin"));
    assert!(!stdout.contains("orphan"));
    assert!(!stdout.contains("0.9.0"));
}

#[test]
fn resolver_order_does_not_change_human_or_json_diagnostics() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true, false));
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n[plugins.\"other@second\"]\nenabled = false\n",
    );
    for (path, name) in [
        (".codex/plugins/cache/marketplace/demo/first", "demo"),
        (".codex/plugins/cache/second/other/second", "other"),
    ] {
        fixture.write_home(
            format!("{path}/.codex-plugin/plugin.json"),
            &format!(r#"{{"name":"{name}","version":"1.0.0"}}"#),
        );
    }
    let first_root = fixture.home().join(".codex/.tmp/first");
    let second_root = fixture.home().join(".codex/.tmp/second");
    let first_marketplace = marketplace("marketplace", &first_root);
    let second_marketplace = marketplace("second", &second_root);
    let first_plugin = plugin(
        "demo@marketplace",
        "marketplace",
        "first",
        true,
        &first_root.join("plugins/demo"),
    );
    let second_plugin = plugin(
        "other@second",
        "second",
        "second",
        false,
        &second_root.join("plugins/other"),
    );
    install(
        &fixture,
        json!({"marketplaces": [second_marketplace.clone(), first_marketplace.clone()]}),
        json!({"installed": [second_plugin.clone(), first_plugin.clone()], "available": []}),
    );
    let args = ["doctor", "skills", "--agent", "codex", "--scope", "user"];
    let (_, first_human, _) = run(&fixture, &args);
    let (_, first_json, _) = run(&fixture, &[&args[..], &["--format", "json"]].concat());

    install(
        &fixture,
        json!({"marketplaces": [first_marketplace, second_marketplace]}),
        json!({"installed": [first_plugin, second_plugin], "available": []}),
    );
    let (_, second_human, _) = run(&fixture, &args);
    let (_, second_json, _) = run(&fixture, &[&args[..], &["--format", "json"]].concat());

    assert_eq!(first_human, second_human);
    assert_eq!(first_json, second_json);
}

#[test]
fn resolver_only_installed_plugin_is_not_dropped_by_the_config_join() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(false, false));
    fixture.write_home(".codex/config.toml", "");
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/revision/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"1.0.0"}"#,
    );
    let root = fixture.home().join(".codex/.tmp/plugins");
    let path = root.join("plugins/demo");
    install(
        &fixture,
        json!({"marketplaces": [marketplace("marketplace", &root)]}),
        json!({"installed": [plugin("demo@marketplace", "marketplace", "revision", true, &path)], "available": []}),
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("DRIFT plugin · enabled · healthy · unexpected"));
}
