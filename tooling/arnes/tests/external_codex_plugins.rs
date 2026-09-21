#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use serde_json::Value;
use skill_support::{MANIFEST, configured_fixture, run};

fn manifest(plugin_allowed: bool) -> String {
    let plugins = if plugin_allowed {
        "    - { agent: codex, scope: user, id: demo@marketplace }\n"
    } else {
        ""
    };
    MANIFEST.replacen(
        "resources:",
        &format!("external:\n  plugins:\n{plugins}resources:"),
        1,
    )
}

#[test]
fn configured_plugins_preserve_exposure_and_policy_without_claiming_availability()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (setting, exposure, activation) in [
        ("enabled = true", "enabled", "unknown"),
        ("enabled = false", "disabled", "disabled"),
        ("", "unknown", "unknown"),
    ] {
        for allowed in [true, false] {
            let fixture = configured_fixture()?;
            fixture.write_home(".arnes.yaml", &manifest(allowed))?;
            fixture.write_home(
                ".codex/config.toml",
                &format!("[plugins.\"demo@marketplace\"]\n{setting}\n"),
            )?;
            let before = fixture.snapshot()?;
            let (code, stdout, stderr) = run(
                &fixture,
                &["doctor", "skills", "--agent", "codex", "--format", "json"],
            )?;
            let unexpected_enabled = exposure == "enabled" && !allowed;
            assert_eq!(code, i32::from(unexpected_enabled), "{stdout}");
            assert!(stderr.is_empty());
            assert_eq!(fixture.snapshot()?, before);
            let diagnostics: Vec<Value> = serde_json::from_str(&stdout)?;
            let plugin = diagnostics
                .iter()
                .find(|diagnostic| {
                    diagnostic
                        .get("message")
                        .and_then(Value::as_str)
                        .is_some_and(|message| message.contains("plugin demo@marketplace "))
                })
                .ok_or("missing configured plugin diagnostic")?;
            assert_eq!(
                plugin.get("state").and_then(Value::as_str),
                Some(if unexpected_enabled {
                    "drift"
                } else {
                    "unsupported"
                }),
            );
            let message = plugin
                .get("message")
                .and_then(Value::as_str)
                .ok_or("missing plugin message")?;
            for expected in [
                format!("exposure={exposure}"),
                format!("activation={activation}"),
                format!("policy={}", if allowed { "allowed" } else { "unexpected" }),
                "topology=unknown".to_owned(),
                "version=unknown".to_owned(),
                "path=unknown".to_owned(),
            ] {
                assert!(message.contains(&expected), "{message}");
            }
            assert!(stdout.contains("read-only"), "{stdout}");
        }
    }
    Ok(())
}

#[test]
fn a_lone_cache_artifact_never_proves_activation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(false))?;
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    )?;
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/0.9.0/.codex-plugin/plugin.json",
        r#"{"name":"demo","version":"0.9.0","skills":["skills"]}"#,
    )?;
    fixture.write_home(
        ".codex/plugins/cache/marketplace/demo/0.9.0/skills/orphan/SKILL.md",
        "orphan\n",
    )?;
    let before = fixture.snapshot()?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("DRIFT plugin · enabled · unknown · unexpected"));
    assert!(stdout.contains("UNSUPPORTED external codex user plugin resolution"));
    assert!(stdout.contains("read-only"));
    assert!(!stdout.contains("orphan"));
    assert!(!stdout.contains("0.9.0"));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn configuration_order_does_not_change_human_or_json_diagnostics()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true))?;
    let first = "[plugins.\"demo@marketplace\"]\nenabled = true\n";
    let second = "[plugins.\"other@second\"]\nenabled = false\n";
    for format in ["human", "json"] {
        fixture.write_home(".codex/config.toml", &format!("{second}{first}"))?;
        let before = fixture.snapshot()?;
        let args = [
            "doctor", "skills", "--agent", "codex", "--scope", "user", "--format", format,
        ];
        let (first_code, first_output, _) = run(&fixture, &args)?;
        assert_eq!(fixture.snapshot()?, before);
        fixture.write_home(".codex/config.toml", &format!("{first}{second}"))?;
        let before = fixture.snapshot()?;
        let (second_code, second_output, _) = run(&fixture, &args)?;
        assert_eq!(first_code, 0);
        assert_eq!(second_code, first_code);
        assert_eq!(first_output, second_output);
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}
