use super::super::support::*;
#[test]
fn cursor_fingerprint_hashes_only_declared_local_plugins()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    fs::write(
        harness.home.join(".arnes.yaml"),
        "version: 1\nagents:\n  - id: cursor\n    scopes: [user]\nexternal:\n  roots: []\n  plugins:\n    - { agent: cursor, scope: user, id: active }\n  skills: []\nresources: []\n",
    )?;
    let active = harness
        .home
        .join(".cursor/plugins/local/active/plugin.json");
    let inactive = harness
        .home
        .join(".cursor/plugins/local/inactive/plugin.json");
    fs::create_dir_all(active.parent().ok_or("fixture path has no parent")?)?;
    fs::create_dir_all(inactive.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&active, "active one")?;
    fs::write(&inactive, "inactive one")?;
    let first = capture_run(&harness, "cursor", "conversation_id", "one")?;
    fs::write(&inactive, "inactive two")?;
    let second = capture_run(&harness, "cursor", "conversation_id", "two")?;
    assert_eq!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    fs::write(&active, "active two")?;
    let third = capture_run(&harness, "cursor", "conversation_id", "three")?;
    assert_ne!(
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(third)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    Ok(())
}
#[test]
fn fingerprint_includes_the_first_512_sorted_deployment_entries()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let skills = harness.home.join(".agents/skills");
    fs::create_dir_all(&skills)?;
    for index in 0..400 {
        fs::write(
            skills.join(format!("skill-{index:03}")),
            format!("value-{index}"),
        )?;
    }
    let first = capture_run(&harness, "codex", "session_id", "one")?;
    fs::write(skills.join("skill-300"), "changed")?;
    let second = capture_run(&harness, "codex", "session_id", "two")?;
    assert_ne!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    Ok(())
}
#[test]
fn fingerprint_bounds_deployments_exceeding_512_entries()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let skills = harness.home.join(".agents/skills");
    fs::create_dir_all(&skills)?;
    for index in 0..513 {
        fs::write(skills.join(format!("skill-{index:03}")), "value")?;
    }
    let first = capture_run(&harness, "codex", "session_id", "one")?;
    assert!(
        (*(first)
            .get("harness_fingerprint_limitations")
            .ok_or("missing fixture index harness_fingerprint_limitations")?)
        .as_array()
        .ok_or("expected JSON array")?
        .iter()
        .map(
            |value| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(value
                    .as_str()
                    .ok_or("expected JSON string")?
                    .contains("inventory"))
            }
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(std::convert::identity)
    );
    fs::write(skills.join("skill-000"), "changed")?;
    let second = capture_run(&harness, "codex", "session_id", "two")?;
    assert_ne!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    Ok(())
}
#[test]
fn fingerprint_bounds_more_than_512_registered_plugin_file_roots()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins)?;
    let mut registered = serde_json::Map::new();
    let mut enabled = serde_json::Map::new();
    for index in 0..513 {
        let plugin = plugins.join(format!("plugin-{index:03}.json"));
        fs::write(&plugin, "plugin")?;
        let id = format!("plugin-{index:03}@marketplace");
        registered.insert(
            id.clone(),
            json ! ([{ "installPath" : plugin , "version" : "1.0.0" }]),
        );
        enabled.insert(id, json!(true));
    }
    fs::create_dir_all(harness.home.join(".claude"))?;
    fs::write(
        harness.home.join(".claude/settings.json"),
        json ! ({ "enabledPlugins" : enabled }).to_string(),
    )?;
    fs::write(
        plugins.join("installed_plugins.json"),
        json ! ({ "version" : 2 , "plugins" : registered }).to_string(),
    )?;
    let run = capture_run(&harness, "claude-code", "session_id", "session")?;
    assert!(
        (*(run)
            .get("harness_fingerprint_limitations")
            .ok_or("missing fixture index harness_fingerprint_limitations")?)
        .as_array()
        .ok_or("expected JSON array")?
        .iter()
        .map(
            |value| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(value
                    .as_str()
                    .ok_or("expected JSON string")?
                    .contains("inventory"))
            }
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(std::convert::identity)
    );
    Ok(())
}
#[test]
fn fingerprint_counts_registered_plugin_aliases_against_the_global_limit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins)?;
    let plugin = plugins.join("shared.json");
    fs::write(&plugin, "plugin")?;
    let mut registered = serde_json::Map::new();
    let mut enabled = serde_json::Map::new();
    for index in 0..513 {
        let id = format!("plugin-{index:03}@marketplace");
        registered.insert(
            id.clone(),
            json ! ([{ "installPath" : plugin , "version" : "1.0.0" }]),
        );
        enabled.insert(id, json!(true));
    }
    fs::create_dir_all(harness.home.join(".claude"))?;
    fs::write(
        harness.home.join(".claude/settings.json"),
        json ! ({ "enabledPlugins" : enabled }).to_string(),
    )?;
    fs::write(
        plugins.join("installed_plugins.json"),
        json ! ({ "version" : 2 , "plugins" : registered }).to_string(),
    )?;
    let run = capture_run(&harness, "claude-code", "session_id", "session")?;
    assert!(
        (*(run)
            .get("harness_fingerprint_limitations")
            .ok_or("missing fixture index harness_fingerprint_limitations")?)
        .as_array()
        .ok_or("expected JSON array")?
        .iter()
        .map(
            |value| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(value
                    .as_str()
                    .ok_or("expected JSON string")?
                    .contains("inventory"))
            }
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(std::convert::identity)
    );
    Ok(())
}
#[test]
fn fingerprint_marks_oversized_plugin_manifests()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins)?;
    let registry = plugins.join("installed_plugins.json");
    fs::write(&registry, vec![b' '; 1_048_577])?;
    let first = capture_run(&harness, "claude-code", "session_id", "one")?;
    assert!(
        (*(first)
            .get("harness_fingerprint_limitations")
            .ok_or("missing fixture index harness_fingerprint_limitations")?)
        .as_array()
        .ok_or("expected JSON array")?
        .iter()
        .map(
            |value| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(value
                    .as_str()
                    .ok_or("expected JSON string")?
                    .contains("manifest"))
            }
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(std::convert::identity)
    );
    let mut file = fs::OpenOptions::new().write(true).open(registry)?;
    file.write_all(b"x")?;
    let second = capture_run(&harness, "claude-code", "session_id", "two")?;
    assert_ne!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    Ok(())
}
