#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use serde_json::Value;
use skill_support::{MANIFEST, configured_fixture, run};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const CACHE: &str = ".codex/plugins/cache/marketplace/demo";

fn manifest(plugin_allowed: bool, allowed_skills: &[&str]) -> String {
    let plugins = if plugin_allowed {
        "    - { agent: codex, scope: user, id: demo@marketplace }\n"
    } else {
        ""
    };
    let skills = allowed_skills
        .iter()
        .map(|slug| {
            format!(
                "    - {{ agent: codex, scope: user, origin: plugin, plugin: demo@marketplace, slug: {slug} }}\n"
            )
        })
        .collect::<Vec<_>>()
        .concat();
    MANIFEST.replacen(
        "resources:",
        &format!("external:\n  plugins:\n{plugins}  skills:\n{skills}resources:"),
        1,
    )
}

fn cached_artifact(fixture: &Fixture, artifact: &str, name: &str, skills: &[&str]) -> TestResult {
    let declared_skills = if skills.is_empty() {
        ""
    } else {
        r#","skills":"./skills/""#
    };
    fixture.write_home(
        format!("{CACHE}/{artifact}/.codex-plugin/plugin.json"),
        &format!(r#"{{"name":"{name}","version":"{artifact}"{declared_skills}}}"#),
    )?;
    for skill in skills {
        fixture.write_home(
            format!("{CACHE}/{artifact}/skills/{skill}/SKILL.md"),
            &format!("# {skill}\n"),
        )?;
    }
    Ok(())
}

fn configure(fixture: &Fixture, id: &str, setting: &str) -> TestResult {
    fixture.write_home(
        ".codex/config.toml",
        &format!("[plugins.\"{id}\"]\n{setting}\n"),
    )
}

fn doctor(
    fixture: &Fixture,
) -> Result<(i32, Vec<Value>), Box<dyn std::error::Error + Send + Sync>> {
    let before = fixture.snapshot()?;
    let (code, stdout, stderr) = run(
        fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "--format", "json",
        ],
    )?;
    assert!(stderr.is_empty(), "{stderr}");
    assert_eq!(fixture.snapshot()?, before);
    assert!(!stdout.contains("plugin resolution"), "{stdout}");
    Ok((code, serde_json::from_str(&stdout)?))
}

fn diagnostic<'a>(
    diagnostics: &'a [Value],
    subject: &str,
) -> Result<(&'a str, &'a str), Box<dyn std::error::Error + Send + Sync>> {
    let found = diagnostics
        .iter()
        .find(|diagnostic| {
            diagnostic
                .get("message")
                .and_then(Value::as_str)
                .is_some_and(|message| message.contains(subject))
        })
        .ok_or_else(|| format!("missing diagnostic for {subject}: {diagnostics:?}"))?;
    Ok((
        found
            .get("state")
            .and_then(Value::as_str)
            .ok_or("missing state")?,
        found
            .get("message")
            .and_then(Value::as_str)
            .ok_or("missing message")?,
    ))
}

#[test]
fn a_single_cached_artifact_is_the_installed_plugin() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &["kept"]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    cached_artifact(&fixture, "0.9.0", "demo", &["kept", "extra"])?;
    let (code, diagnostics) = doctor(&fixture)?;
    assert_eq!(code, 1);
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "healthy", "{message}");
    for expected in [
        "version=0.9.0",
        "artifact=0.9.0",
        "exposure=enabled",
        "topology=healthy",
        "policy=allowed",
        "activation=available-not-runtime-observed",
    ] {
        assert!(message.contains(expected), "{message}");
    }
    assert_eq!(diagnostic(&diagnostics, "skill kept ")?.0, "healthy");
    assert_eq!(diagnostic(&diagnostics, "skill extra ")?.0, "drift");
    Ok(())
}

#[test]
fn an_enabled_plugin_outside_policy_drifts() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(false, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    cached_artifact(&fixture, "0.9.0", "demo", &[])?;
    let (code, diagnostics) = doctor(&fixture)?;
    assert_eq!(code, 1);
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "drift", "{message}");
    assert!(message.contains("policy=unexpected"), "{message}");
    Ok(())
}

#[test]
fn a_latest_link_to_the_cached_version_is_the_same_artifact() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    cached_artifact(&fixture, "0.9.0", "demo", &[])?;
    symlink(
        fixture.home().join(CACHE).join("0.9.0"),
        fixture.home().join(CACHE).join("latest"),
    )?;
    let (code, diagnostics) = doctor(&fixture)?;
    assert_eq!(code, 0);
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "healthy", "{message}");
    assert!(message.contains("artifact=0.9.0"), "{message}");
    Ok(())
}

#[test]
fn a_configured_plugin_without_cached_artifact_is_missing() -> TestResult {
    for (setting, expected_state, expected_code) in [
        ("enabled = true", "drift", 1),
        ("enabled = false", "healthy", 0),
        ("", "unsupported", 0),
    ] {
        let fixture = configured_fixture()?;
        fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
        configure(&fixture, "demo@marketplace", setting)?;
        fixture.write_home(format!("{CACHE}/.codex-remote-plugin-install.json"), "{}")?;
        fixture.write_home(format!("{CACHE}/.tmpStaging/plugin.json"), "{}")?;
        fixture.write_home(format!("{CACHE}/notes.txt"), "not an artifact\n")?;
        let (code, diagnostics) = doctor(&fixture)?;
        assert_eq!(code, expected_code, "{setting}");
        let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
        assert_eq!(state, expected_state, "{message}");
        assert!(message.contains("topology=missing"), "{message}");
    }
    Ok(())
}

#[test]
fn several_cached_artifacts_leave_the_active_one_unobservable() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    cached_artifact(&fixture, "0.9.0", "demo", &[])?;
    cached_artifact(&fixture, "1.0.0", "demo", &[])?;
    let (code, diagnostics) = doctor(&fixture)?;
    assert_eq!(code, 0);
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "unsupported", "{message}");
    for expected in ["topology=unknown", "activation=unknown", "0.9.0, 1.0.0"] {
        assert!(message.contains(expected), "{message}");
    }
    Ok(())
}

#[test]
fn a_cached_artifact_must_stay_inside_its_plugin_cache_directory() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    fixture.write_home("elsewhere/.codex-plugin/plugin.json", r#"{"name":"demo"}"#)?;
    fixture.write_home(format!("{CACHE}/.keep"), "")?;
    symlink(
        fixture.home().join("elsewhere"),
        fixture.home().join(CACHE).join("0.9.0"),
    )?;
    let (code, diagnostics) = doctor(&fixture)?;
    assert_eq!(code, 2);
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "error", "{message}");
    assert!(message.contains("topology=broken"), "{message}");
    Ok(())
}

#[test]
fn a_cached_manifest_must_name_the_configured_plugin() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    cached_artifact(&fixture, "0.9.0", "other", &[])?;
    let (code, diagnostics) = doctor(&fixture)?;
    assert_eq!(code, 2);
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "error", "{message}");
    assert!(message.contains("topology=broken"), "{message}");
    Ok(())
}

#[test]
fn an_identifier_without_marketplace_cannot_be_located() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(false, &[]))?;
    configure(&fixture, "demo", "enabled = false")?;
    let (code, diagnostics) = doctor(&fixture)?;
    assert_eq!(code, 0);
    let (state, message) = diagnostic(&diagnostics, "plugin demo ")?;
    assert_eq!(state, "unsupported", "{message}");
    assert!(message.contains("topology=unknown"), "{message}");
    Ok(())
}

#[test]
fn configuration_order_does_not_change_human_or_json_diagnostics() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    cached_artifact(&fixture, "0.9.0", "demo", &[])?;
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
        assert_eq!(first_code, 0, "{first_output}");
        assert_eq!(second_code, first_code);
        assert_eq!(first_output, second_output);
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

fn assert_broken_cache(fixture: &Fixture, detail: &str) -> TestResult {
    let (code, diagnostics) = doctor(fixture)?;
    assert_eq!(code, 2);
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "error", "{message}");
    assert!(message.contains("topology=broken"), "{message}");
    assert!(message.contains(detail), "{message}");
    Ok(())
}

#[test]
fn a_dangling_cached_artifact_link_is_broken() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    cached_artifact(&fixture, "0.9.0", "demo", &[])?;
    symlink(
        fixture.home().join(CACHE).join("absent"),
        fixture.home().join(CACHE).join("latest"),
    )?;
    assert_broken_cache(&fixture, "cached artifact link is dangling")
}

#[test]
fn a_cached_artifact_aliasing_a_nested_path_is_broken() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    cached_artifact(&fixture, "0.9.0", "demo", &[])?;
    fixture.write_home(
        format!("{CACHE}/0.9.0/nested/.codex-plugin/plugin.json"),
        r#"{"name":"demo"}"#,
    )?;
    symlink(
        fixture.home().join(CACHE).join("0.9.0/nested"),
        fixture.home().join(CACHE).join("latest"),
    )?;
    assert_broken_cache(&fixture, "aliases a path nested inside")
}

#[test]
fn a_plugin_cache_directory_outside_the_codex_cache_is_broken() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    fixture.write_home(
        "elsewhere/demo/0.9.0/.codex-plugin/plugin.json",
        r#"{"name":"demo"}"#,
    )?;
    fs::create_dir_all(fixture.home().join(".codex/plugins/cache"))?;
    symlink(
        fixture.home().join("elsewhere"),
        fixture.home().join(".codex/plugins/cache/marketplace"),
    )?;
    assert_broken_cache(&fixture, "resolves outside the Codex plugin cache")
}

#[test]
fn an_unreadable_plugin_cache_directory_is_reported() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, &[]))?;
    configure(&fixture, "demo@marketplace", "enabled = true")?;
    let directory = fixture.home().join(CACHE);
    fs::create_dir_all(&directory)?;
    let before = fixture.snapshot()?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o000))?;
    let output = fixture.command([
        "doctor", "skills", "--agent", "codex", "--scope", "user", "--format", "json",
    ])?;
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o755))?;
    assert_eq!(fixture.snapshot()?, before);
    assert_eq!(output.status.code(), Some(2));
    let diagnostics: Vec<Value> = serde_json::from_slice(&output.stdout)?;
    let (state, message) = diagnostic(&diagnostics, "plugin demo@marketplace ")?;
    assert_eq!(state, "error", "{message}");
    assert!(message.contains("topology=unreadable"), "{message}");
    Ok(())
}
