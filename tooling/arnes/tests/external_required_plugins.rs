#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use serde_json::json;
use skill_support::{MANIFEST, configured_fixture, run};
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const AGENTS: [&str; 3] = ["claude", "codex", "cursor"];

fn plugin_id(agent: &str) -> &str {
    if agent == "cursor" {
        "agent-reach"
    } else {
        "agent-reach@agent-reach-adapter"
    }
}

fn manifest(agent: &str, requirement: &str) -> String {
    let id = plugin_id(agent);
    MANIFEST.replacen(
        "resources:",
        &format!(
            "external:\n  plugins:\n    - {{ agent: {agent}, scope: user, id: {id}{requirement} }}\n  skills:\n    - {{ agent: {agent}, scope: user, origin: plugin, plugin: {id}, slug: agent-reach }}\nresources:"
        ),
        1,
    )
}

fn install(fixture: &Fixture, agent: &str, enabled: bool) -> TestResult {
    let id = plugin_id(agent);
    let root = if agent == "cursor" {
        ".cursor/plugins/local/agent-reach".to_owned()
    } else {
        format!(".{agent}/plugins/cache/agent-reach-adapter/agent-reach/0.0.1")
    };
    fixture.write_home(
        format!("{root}/.{agent}-plugin/plugin.json"),
        r#"{"name":"agent-reach","version":"0.0.1","skills":"./skills"}"#,
    )?;
    fixture.write_home(
        format!("{root}/skills/agent-reach/SKILL.md"),
        "# Agent Reach\n",
    )?;
    if agent == "claude" {
        fixture.write_home(
            ".claude/settings.json",
            &json!({ "enabledPlugins": { id: enabled } }).to_string(),
        )?;
        fixture.write_home(
            ".claude/plugins/installed_plugins.json",
            &json!({
                "version": 2,
                "plugins": { id: [{
                    "scope": "user",
                    "installPath": fixture.home().join(root),
                    "version": "0.0.1",
                    "installedAt": "2026-10-19T00:00:00Z"
                }] }
            })
            .to_string(),
        )?;
    } else if agent == "codex" {
        fixture.write_home(
            ".codex/config.toml",
            &format!("[plugins.\"{id}\"]\nenabled = {enabled}\n"),
        )?;
    }
    Ok(())
}

fn doctor(
    fixture: &Fixture,
    agent: &str,
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    run(
        fixture,
        &[
            "doctor", "skills", "--agent", agent, "--scope", "user", "--format", "json",
        ],
    )
}

#[test]
fn required_plugins_accept_enabled_installations_with_observable_activation() -> TestResult {
    for agent in ["claude", "codex"] {
        let fixture = configured_fixture()?;
        fixture.write_home(".arnes.yaml", &manifest(agent, ", required: true"))?;
        install(&fixture, agent, true)?;
        let (code, stdout, stderr) = doctor(&fixture, agent)?;
        assert_eq!(code, 0, "{agent}: {stdout}\n{stderr}");
        assert!(
            stdout.contains("activation=available-not-runtime-observed"),
            "{agent}: {stdout}"
        );
    }
    Ok(())
}

#[test]
fn a_required_cursor_projection_does_not_prove_local_import_authorization() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest("cursor", ", required: true"))?;
    install(&fixture, "cursor", true)?;
    let (code, stdout, stderr) = doctor(&fixture, "cursor")?;
    assert_eq!(code, 2, "{stdout}\n{stderr}");
    assert!(stdout.contains("exposure=unknown"), "{stdout}");
    assert!(stdout.contains("local import authorization"), "{stdout}");
    Ok(())
}

#[test]
fn required_plugins_drift_when_the_installation_is_absent() -> TestResult {
    for agent in AGENTS {
        let fixture = configured_fixture()?;
        fixture.write_home(".arnes.yaml", &manifest(agent, ", required: true"))?;
        let (code, stdout, stderr) = doctor(&fixture, agent)?;
        assert_eq!(code, 1, "{agent}: {stdout}\n{stderr}");
        assert!(
            stdout.contains("required plugin is not installed"),
            "{agent}: {stdout}"
        );
    }
    Ok(())
}

#[test]
fn required_plugins_drift_when_installed_but_disabled() -> TestResult {
    for agent in ["claude", "codex"] {
        let fixture = configured_fixture()?;
        fixture.write_home(".arnes.yaml", &manifest(agent, ", required: true"))?;
        install(&fixture, agent, false)?;
        let (code, stdout, stderr) = doctor(&fixture, agent)?;
        assert_eq!(code, 1, "{agent}: {stdout}\n{stderr}");
        assert!(stdout.contains("exposure=disabled"), "{agent}: {stdout}");
        assert!(
            stdout.contains("required plugin is disabled"),
            "{agent}: {stdout}"
        );
    }
    Ok(())
}

#[test]
fn optional_plugins_remain_optional_when_absent_or_disabled() -> TestResult {
    for agent in AGENTS {
        for requirement in ["", ", required: false"] {
            let fixture = configured_fixture()?;
            fixture.write_home(".arnes.yaml", &manifest(agent, requirement))?;
            let (code, stdout, stderr) = doctor(&fixture, agent)?;
            assert_eq!(code, 0, "{agent}: {stdout}\n{stderr}");
            if agent != "cursor" {
                install(&fixture, agent, false)?;
                let (code, stdout, stderr) = doctor(&fixture, agent)?;
                assert_eq!(code, 0, "{agent}: {stdout}\n{stderr}");
            }
        }
    }
    Ok(())
}

#[test]
fn a_required_codex_plugin_with_unknown_activation_fails_closed() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest("codex", ", required: true"))?;
    install(&fixture, "codex", true)?;
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"agent-reach@agent-reach-adapter\"]\n",
    )?;
    let (code, stdout, stderr) = doctor(&fixture, "codex")?;
    assert_eq!(code, 2, "{stdout}\n{stderr}");
    assert!(stdout.contains("exposure=unknown"), "{stdout}");
    Ok(())
}

#[test]
fn a_required_codex_plugin_with_ambiguous_cached_artifacts_fails_closed() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest("codex", ", required: true"))?;
    install(&fixture, "codex", true)?;
    fixture.write_home(
        ".codex/plugins/cache/agent-reach-adapter/agent-reach/0.0.2/.codex-plugin/plugin.json",
        r#"{"name":"agent-reach","version":"0.0.2"}"#,
    )?;
    let (code, stdout, stderr) = doctor(&fixture, "codex")?;
    assert_eq!(code, 2, "{stdout}\n{stderr}");
    assert!(stdout.contains("topology=unknown"), "{stdout}");
    Ok(())
}

#[test]
fn requiring_a_plugin_does_not_allow_undeclared_skills() -> TestResult {
    for agent in AGENTS {
        let fixture = configured_fixture()?;
        fixture.write_home(".arnes.yaml", &manifest(agent, ", required: true"))?;
        install(&fixture, agent, true)?;
        let root = if agent == "cursor" {
            ".cursor/plugins/local/agent-reach".to_owned()
        } else {
            format!(".{agent}/plugins/cache/agent-reach-adapter/agent-reach/0.0.1")
        };
        fixture.write_home(format!("{root}/skills/foreign/SKILL.md"), "# Foreign\n")?;
        let (code, stdout, stderr) = doctor(&fixture, agent)?;
        assert_eq!(
            code,
            if agent == "cursor" { 2 } else { 1 },
            "{agent}: {stdout}\n{stderr}"
        );
        assert!(stdout.contains("skill foreign"), "{agent}: {stdout}");
        assert!(stdout.contains("policy=unexpected"), "{agent}: {stdout}");
    }
    Ok(())
}

#[test]
fn a_required_plugin_respects_the_selected_agent_and_scope() -> TestResult {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest("claude", ", required: true"))?;
    let (code, stdout, stderr) = doctor(&fixture, "codex")?;
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(!stdout.contains("required plugin"), "{stdout}");
    fixture.write_repository(".agents/skills/beta/references/missing.md", "guide\n")?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "claude", "--scope", "project",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}\n{stderr}");
    assert!(!stdout.contains("required plugin"), "{stdout}");
    Ok(())
}

#[test]
fn required_plugins_reject_unobservable_project_scopes() -> TestResult {
    for agent in ["codex", "cursor"] {
        let fixture = configured_fixture()?;
        let policy = MANIFEST.replacen("resources:", &format!("external:\n  plugins:\n    - {{ agent: {agent}, scope: project, id: example, required: true }}\nresources:"), 1);
        fixture.write_home(".arnes.yaml", &policy)?;
        let (code, stdout, stderr) = doctor(&fixture, agent)?;
        assert_eq!(code, 2, "{agent}: {stdout}\n{stderr}");
        assert!(
            stdout.contains("required plugins need an observable scope"),
            "{agent}: {stdout}"
        );
    }
    Ok(())
}

#[test]
fn plugin_requirements_reject_non_boolean_values() -> TestResult {
    for value in ["null", "sometimes", "[]"] {
        let fixture = configured_fixture()?;
        fixture.write_home(
            ".arnes.yaml",
            &manifest("claude", &format!(", required: {value}")),
        )?;
        let (code, stdout, stderr) = doctor(&fixture, "claude")?;
        assert_eq!(code, 2, "{stdout}\n{stderr}");
        assert!(stdout.contains("required"), "{stdout}");
    }
    Ok(())
}
