#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;
use serde_json::Value;
use skill_support::{MANIFEST, configured_fixture, run};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use support::Fixture;
fn manifest(allow_plugin: bool, allow_skill: bool, system: bool) -> String {
    let roots = if system {
        "    - agent: cursor\n      scope: user\n      origin: system\n      location: { root: home, path: .cursor/system-skills }\n"
    } else {
        ""
    };
    let plugins = if allow_plugin {
        "    - { agent: cursor, scope: user, id: cursor-demo }\n"
    } else {
        ""
    };
    let mut skills = String::new();
    if allow_skill {
        skills . push_str ("    - { agent: cursor, scope: user, origin: plugin, plugin: cursor-demo, slug: hello }\n" ,) ;
    }
    if system {
        skills.push_str("    - { agent: cursor, scope: user, origin: system, slug: alpha }\n");
    }
    MANIFEST.replacen(
        "resources:",
        &format!("external:\n  roots:\n{roots}  plugins:\n{plugins}  skills:\n{skills}resources:"),
        1,
    )
}
fn plugin(
    fixture: &Fixture,
    directory: &str,
    id: &str,
    slug: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = format!(".cursor/plugins/local/{directory}");
    fixture.write_home(
        format!("{root}/plugin.json"),
        &format!("{{\"name\":\"{id}\",\"version\":\"1.2.3\",\"skills\":\"./skills\"}}"),
    )?;
    fixture.write_home(format!("{root}/skills/{slug}/SKILL.md"), "skill\n")?;
    Ok(())
}
#[test]
fn active_allowlisted_cursor_plugin_and_skill_are_healthy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, true, false))?;
    plugin(&fixture, "demo-directory", "cursor-demo", "hello")?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("cursor user plugin cursor-demo@1.2.3"));
    assert!(stdout.contains("plugin · enabled · healthy · allowed"));
    assert!(stdout.contains("skill hello · enabled · healthy · allowed"));
    Ok(())
}
#[test]
fn active_unexpected_cursor_plugin_reports_plugin_and_skill()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(false, false, false))?;
    plugin(&fixture, "demo-directory", "cursor-demo", "hello")?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "cursor", "--scope", "user"],
    )?;
    assert_eq!(code, 1, "{stdout}");
    assert_eq!(stdout.matches("    DRIFT ").count(), 2);
    assert!(stdout.contains("plugin cursor-demo"));
    assert!(stdout.contains("skill hello"));
    Ok(())
}
#[test]
fn managed_system_and_plugin_slug_collisions_remain_distinct()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let policy = manifest (true , false , true) . replace ("  skills:\n    - { agent: cursor, scope: user, origin: system, slug: alpha }" , "  skills:\n    - { agent: cursor, scope: user, origin: plugin, plugin: cursor-demo, slug: alpha }\n    - { agent: cursor, scope: user, origin: system, slug: alpha }" ,) ;
    fixture.write_home(".arnes.yaml", &policy)?;
    plugin(&fixture, "demo-directory", "cursor-demo", "alpha")?;
    fixture.write_home(".cursor/system-skills/alpha/SKILL.md", "system\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("HEALTHY alpha"));
    assert!(stdout.contains("cursor user system skills"));
    assert!(stdout.contains("alpha · enabled · healthy · allowed"));
    assert!(stdout.contains("skill alpha · enabled · healthy · allowed"));
    Ok(())
}
#[test]
fn human_and_json_plugin_order_is_stable_and_filters_are_preserved()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(false, false, false))?;
    plugin(&fixture, "z-directory", "z-plugin", "z-skill")?;
    plugin(&fixture, "a-directory", "a-plugin", "a-skill")?;
    let (_, human, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "cursor", "--scope", "user"],
    )?;
    let (_, json, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "user", "--format", "json",
        ],
    )?;
    assert!(
        human
            .find("plugin a-plugin")
            .ok_or("required test value is missing")?
            < human
                .find("plugin z-plugin")
                .ok_or("required test value is missing")?
    );
    let diagnostics: Vec<Value> = serde_json::from_str(&json)?;
    let messages = diagnostics
        .iter()
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok((*(diagnostic)
                    .get("message")
                    .ok_or("missing fixture index message")?)
                .as_str()
                .ok_or("expected JSON string")?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<Vec<_>>();
    let a = messages
        .iter()
        .position(|message| message.contains("plugin a-plugin"))
        .ok_or("required test value is missing")?;
    let z = messages
        .iter()
        .position(|message| message.contains("plugin z-plugin"))
        .ok_or("required test value is missing")?;
    assert!(a < z);
    assert!(
        messages
            .iter()
            .all(|message| message.contains("cursor user"))
    );
    Ok(())
}
#[test]
fn duplicate_plugin_ids_and_skill_slugs_are_explicit_errors()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(true, true, false))?;
    plugin(&fixture, "first", "cursor-demo", "hello")?;
    plugin(&fixture, "second", "cursor-demo", "hello")?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "cursor", "--scope", "user"],
    )?;
    assert_eq!(code, 2, "{stdout}");
    assert_eq!(stdout.matches("duplicate plugin identifier").count(), 2);
    assert_eq!(stdout.matches("duplicate plugin skill slug").count(), 2);
    Ok(())
}
#[test]
fn invalid_cursor_plugin_manifest_is_topological_error()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(false, false, false))?;
    fixture.write_home(".cursor/plugins/local/broken/plugin.json", "not JSON\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "cursor", "--scope", "user"],
    )?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("plugin broken"));
    assert!(stdout.contains("· broken · unexpected"));
    assert!(stdout.contains("plugin manifest is invalid JSON"));
    Ok(())
}
#[test]
fn unreadable_cursor_plugin_is_distinct_from_policy_and_read_only()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(false, false, false))?;
    plugin(&fixture, "unreadable", "cursor-demo", "hello")?;
    let plugin_manifest = fixture
        .home()
        .join(".cursor/plugins/local/unreadable/plugin.json");
    let before = fixture.snapshot()?;
    fs::set_permissions(&plugin_manifest, fs::Permissions::from_mode(0o000))?;
    let output = fixture.command(["doctor", "skills", "--agent", "cursor", "--scope", "user"])?;
    fs::set_permissions(&plugin_manifest, fs::Permissions::from_mode(0o644))?;
    assert_eq!(fixture.snapshot()?, before);
    assert_eq!(output.status.code(), Some(2));
    let stdout = String::from_utf8(output.stdout)?;
    assert!(stdout.contains("· unreadable · unexpected"));
    assert!(stdout.contains("plugin manifest could not be read"));
    Ok(())
}
