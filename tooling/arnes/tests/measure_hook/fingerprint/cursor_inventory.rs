use super::super::support::*;

#[test]
fn cursor_fingerprint_hashes_only_declared_local_plugins() {
    let harness = Harness::new();
    fs::write(
        harness.home.join(".arnes.yaml"),
        "version: 1\nagents:\n  - id: cursor\n    scopes: [user]\nexternal:\n  roots: []\n  plugins:\n    - { agent: cursor, scope: user, id: active }\n  skills: []\nresources: []\n",
    )
    .unwrap();
    let active = harness
        .home
        .join(".cursor/plugins/local/active/plugin.json");
    let inactive = harness
        .home
        .join(".cursor/plugins/local/inactive/plugin.json");
    fs::create_dir_all(active.parent().unwrap()).unwrap();
    fs::create_dir_all(inactive.parent().unwrap()).unwrap();
    fs::write(&active, "active one").unwrap();
    fs::write(&inactive, "inactive one").unwrap();

    let first = capture_run(&harness, "cursor", "conversation_id", "one");
    fs::write(&inactive, "inactive two").unwrap();
    let second = capture_run(&harness, "cursor", "conversation_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(&active, "active two").unwrap();
    let third = capture_run(&harness, "cursor", "conversation_id", "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);
}

#[test]
fn fingerprint_includes_the_first_512_sorted_deployment_entries() {
    let harness = Harness::new();
    let skills = harness.home.join(".agents/skills");
    fs::create_dir_all(&skills).unwrap();
    for index in 0..400 {
        fs::write(
            skills.join(format!("skill-{index:03}")),
            format!("value-{index}"),
        )
        .unwrap();
    }
    let first = capture_run(&harness, "codex", "session_id", "one");
    fs::write(skills.join("skill-300"), "changed").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");

    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn fingerprint_bounds_deployments_exceeding_512_entries() {
    let harness = Harness::new();
    let skills = harness.home.join(".agents/skills");
    fs::create_dir_all(&skills).unwrap();
    for index in 0..513 {
        fs::write(skills.join(format!("skill-{index:03}")), "value").unwrap();
    }

    let first = capture_run(&harness, "codex", "session_id", "one");
    assert!(
        first["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("inventory"))
    );
    fs::write(skills.join("skill-000"), "changed").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");
    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn fingerprint_bounds_more_than_512_registered_plugin_file_roots() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins).unwrap();
    let mut registered = serde_json::Map::new();
    let mut enabled = serde_json::Map::new();
    for index in 0..513 {
        let plugin = plugins.join(format!("plugin-{index:03}.json"));
        fs::write(&plugin, "plugin").unwrap();
        let id = format!("plugin-{index:03}@marketplace");
        registered.insert(
            id.clone(),
            json!([{"installPath":plugin,"version":"1.0.0"}]),
        );
        enabled.insert(id, json!(true));
    }
    fs::create_dir_all(harness.home.join(".claude")).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        json!({"enabledPlugins":enabled}).to_string(),
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({"version":2,"plugins":registered}).to_string(),
    )
    .unwrap();

    let run = capture_run(&harness, "claude-code", "session_id", "session");
    assert!(
        run["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("inventory"))
    );
}

#[test]
fn fingerprint_counts_registered_plugin_aliases_against_the_global_limit() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins).unwrap();
    let plugin = plugins.join("shared.json");
    fs::write(&plugin, "plugin").unwrap();
    let mut registered = serde_json::Map::new();
    let mut enabled = serde_json::Map::new();
    for index in 0..513 {
        let id = format!("plugin-{index:03}@marketplace");
        registered.insert(
            id.clone(),
            json!([{"installPath":plugin,"version":"1.0.0"}]),
        );
        enabled.insert(id, json!(true));
    }
    fs::create_dir_all(harness.home.join(".claude")).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        json!({"enabledPlugins":enabled}).to_string(),
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({"version":2,"plugins":registered}).to_string(),
    )
    .unwrap();

    let run = capture_run(&harness, "claude-code", "session_id", "session");
    assert!(
        run["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("inventory"))
    );
}

#[test]
fn fingerprint_marks_oversized_plugin_manifests() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    fs::create_dir_all(&plugins).unwrap();
    let registry = plugins.join("installed_plugins.json");
    fs::write(&registry, vec![b' '; 1_048_577]).unwrap();
    let first = capture_run(&harness, "claude-code", "session_id", "one");
    assert!(
        first["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("manifest"))
    );
    let mut file = fs::OpenOptions::new().write(true).open(registry).unwrap();
    file.write_all(b"x").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");
    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}
