#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use skill_support::{MANIFEST, configured_fixture, run};

fn manifest(allowed: bool) -> String {
    let plugins = if allowed {
        "    - { agent: codex, scope: user, id: demo@marketplace }\n"
    } else {
        ""
    };
    MANIFEST.replacen(
        "resources:",
        &format!(
            "external:\n  roots:\n    - {{ agent: codex, scope: user, origin: system, location: {{ root: home, path: .codex/skills/.system }} }}\n  plugins:\n{plugins}  skills: []\nresources:"
        ),
        1,
    )
}

#[test]
fn codex_plugin_policy_uses_config_but_never_guesses_active_cache_version() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(false));
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    );
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/0.9.0/skills/orphan/SKILL.md",
        "orphan\n",
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("drift skills: external codex user plugin demo@marketplace"));
    assert!(stdout.contains("exposure=enabled topology=unknown policy=unexpected"));
    assert!(stdout.contains("active cache version cannot be selected reliably"));
    assert!(!stdout.contains("orphan"));
    assert!(!stdout.contains("0.9.0"));
}

#[test]
fn allowed_or_disabled_codex_plugin_does_not_create_false_policy_drift() {
    let fixture = configured_fixture();
    fixture.write_home(".arnes.yaml", &manifest(true));
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    );
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("unsupported skills: external codex user plugin demo@marketplace"));
    assert!(stdout.contains("policy=allowed"));

    fixture.write_home(".arnes.yaml", &manifest(false));
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = false\n",
    );
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("exposure=disabled topology=unknown policy=unexpected"));
    assert!(!stdout.contains("drift skills: external codex user plugin"));
}
