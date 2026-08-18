use super::*;

#[test]
fn fingerprint_tracks_project_instructions_config_hooks_and_skills() {
    let harness = Harness::new();
    fs::write(harness.repository.join("AGENTS.md"), "first instructions").unwrap();
    fs::create_dir(harness.repository.join(".codex")).unwrap();
    fs::write(
        harness.repository.join(".codex/config.toml"),
        "project='first'",
    )
    .unwrap();
    fs::write(
        harness.repository.join(".codex/hooks.json"),
        r#"{"hooks":{}}"#,
    )
    .unwrap();
    fs::create_dir_all(harness.repository.join(".codex/skills/example")).unwrap();
    fs::write(
        harness.repository.join(".codex/skills/example/SKILL.md"),
        "first skill",
    )
    .unwrap();
    assert_success(&harness.run("codex", br#"{"session_id":"one","event":"SessionStart"}"#));
    let first = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "one")
        .unwrap();

    fs::write(harness.repository.join("AGENTS.md"), "second instructions").unwrap();
    assert_success(&harness.run("codex", br#"{"session_id":"two","event":"SessionStart"}"#));
    let second = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "two")
        .unwrap();

    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
    assert!(!first.to_string().contains("first instructions"));
}

#[test]
fn claude_fingerprint_hashes_only_enabled_registered_plugins() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0/plugin.json");
    let disabled = plugins.join("cache/marketplace/disabled/1.0.0/plugin.json");
    fs::create_dir_all(active.parent().unwrap()).unwrap();
    fs::create_dir_all(disabled.parent().unwrap()).unwrap();
    fs::write(&active, "active one").unwrap();
    fs::write(&disabled, "disabled one").unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true,"disabled@marketplace":false}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":2,
            "plugins":{
                "active@marketplace":[{"scope":"user","installPath":active.parent().unwrap(),"version":"1.0.0"}],
                "disabled@marketplace":[{"scope":"user","installPath":disabled.parent().unwrap(),"version":"1.0.0"}]
            }
        })
        .to_string(),
    )
    .unwrap();

    let first = capture_run(&harness, "claude-code", "session_id", "one");
    fs::write(&disabled, "disabled two").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(&active, "active two").unwrap();
    let third = capture_run(&harness, "claude-code", "session_id", "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);
}

#[test]
fn claude_fingerprint_does_not_follow_active_plugin_symlinks_outside_the_installation() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active).unwrap();
    fs::write(active.join("plugin.json"), "plugin").unwrap();
    let transcript = harness.home.join("session.jsonl");
    fs::write(&transcript, "session one").unwrap();
    symlink(&transcript, active.join("session.jsonl")).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":2,
            "plugins":{"active@marketplace":[{"scope":"user","installPath":active,"version":"1.0.0"}]}
        })
        .to_string(),
    )
    .unwrap();

    let first = capture_run(&harness, "claude-code", "session_id", "one");
    fs::write(transcript, "session two").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");

    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn claude_fingerprint_windows_oversized_active_plugin_files() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active).unwrap();
    let plugin = active.join("plugin.bin");
    fs::write(&plugin, vec![b'x'; 1_048_577]).unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":2,
            "plugins":{"active@marketplace":[{"scope":"user","installPath":active,"version":"1.0.0"}]}
        })
        .to_string(),
    )
    .unwrap();

    let first = capture_run(&harness, "claude-code", "session_id", "one");
    assert!(
        first["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("window"))
    );
    let mut file = fs::OpenOptions::new().write(true).open(plugin).unwrap();
    file.seek(SeekFrom::End(-1)).unwrap();
    file.write_all(b"y").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");
    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn unsupported_claude_registry_excludes_plugin_files_with_a_limitation() {
    let harness = Harness::new();
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active).unwrap();
    fs::write(active.join("plugin.json"), "plugin one").unwrap();
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )
    .unwrap();
    fs::write(
        plugins.join("installed_plugins.json"),
        json!({
            "version":3,
            "plugins":{"active@marketplace":[{"scope":"user","installPath":active,"version":"1.0.0"}]}
        })
        .to_string(),
    )
    .unwrap();

    let first = capture_run(&harness, "claude-code", "session_id", "one");
    assert!(
        first["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("registry"))
    );
    fs::write(active.join("plugin.json"), "plugin two").unwrap();
    let second = capture_run(&harness, "claude-code", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn codex_fingerprint_hashes_only_single_version_enabled_plugins() {
    let harness = Harness::new();
    fs::create_dir_all(harness.home.join(".codex")).unwrap();
    fs::write(
        harness.home.join(".codex/config.toml"),
        "[plugins.\"active@marketplace\"]\nenabled = true\n[plugins.\"disabled@marketplace\"]\nenabled = false\n",
    )
    .unwrap();
    let active = harness
        .home
        .join(".codex/plugins/cache/marketplace/active/1.0.0/plugin.json");
    let disabled = harness
        .home
        .join(".codex/plugins/cache/marketplace/disabled/1.0.0/plugin.json");
    fs::create_dir_all(active.parent().unwrap()).unwrap();
    fs::create_dir_all(disabled.parent().unwrap()).unwrap();
    fs::write(&active, "active one").unwrap();
    fs::write(&disabled, "disabled one").unwrap();

    let first = capture_run(&harness, "codex", "session_id", "one");
    fs::write(&disabled, "disabled two").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(&active, "active two").unwrap();
    let third = capture_run(&harness, "codex", "session_id", "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);
}

#[test]
fn codex_fingerprint_records_ambiguous_enabled_plugin_versions() {
    let harness = Harness::new();
    fs::create_dir_all(harness.home.join(".codex")).unwrap();
    fs::write(
        harness.home.join(".codex/config.toml"),
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    )
    .unwrap();
    let cache = harness.home.join(".codex/plugins/cache/marketplace/demo");
    for version in ["1.0.0", "2.0.0"] {
        fs::create_dir_all(cache.join(version)).unwrap();
        fs::write(cache.join(version).join("plugin.json"), version).unwrap();
    }

    let first = capture_run(&harness, "codex", "session_id", "one");
    assert!(
        first["harness_fingerprint_limitations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("ambiguous"))
    );
    fs::create_dir_all(cache.join("3.0.0")).unwrap();
    fs::write(cache.join("3.0.0/plugin.json"), "three").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");
    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

#[test]
fn codex_ambiguous_version_sets_have_unambiguous_fingerprints() {
    let capture = |versions: &[&str]| {
        let harness = Harness::new();
        fs::create_dir_all(harness.home.join(".codex")).unwrap();
        fs::write(
            harness.home.join(".codex/config.toml"),
            "[plugins.\"demo@marketplace\"]\nenabled = true\n",
        )
        .unwrap();
        let cache = harness.home.join(".codex/plugins/cache/marketplace/demo");
        for version in versions {
            fs::create_dir_all(cache.join(version)).unwrap();
        }
        capture_run(&harness, "codex", "session_id", "session")["harness_fingerprint"]
            .as_str()
            .unwrap()
            .to_owned()
    };

    assert_ne!(capture(&["a,b", "c"]), capture(&["a", "b,c"]));
}

#[test]
fn codex_plugin_roots_cannot_escape_the_cache() {
    let escaped = Harness::new();
    fs::create_dir_all(escaped.home.join(".codex")).unwrap();
    fs::write(
        escaped.home.join(".codex/config.toml"),
        "[plugins.\"payload@../../../outside\"]\nenabled = true\n",
    )
    .unwrap();
    let escaped_file = escaped.home.join("outside/payload/1.0.0/plugin.json");
    fs::create_dir_all(escaped_file.parent().unwrap()).unwrap();
    fs::write(&escaped_file, "one").unwrap();
    let first = capture_run(&escaped, "codex", "session_id", "one");
    fs::write(&escaped_file, "two").unwrap();
    let second = capture_run(&escaped, "codex", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);

    let linked = Harness::new();
    fs::create_dir_all(linked.home.join(".codex/plugins/cache/marketplace")).unwrap();
    fs::write(
        linked.home.join(".codex/config.toml"),
        "[plugins.\"active@marketplace\"]\nenabled = true\n",
    )
    .unwrap();
    let outside = linked.home.join("outside/active");
    let linked_file = outside.join("1.0.0/plugin.json");
    fs::create_dir_all(linked_file.parent().unwrap()).unwrap();
    fs::write(&linked_file, "one").unwrap();
    symlink(
        &outside,
        linked.home.join(".codex/plugins/cache/marketplace/active"),
    )
    .unwrap();
    let first = capture_run(&linked, "codex", "session_id", "one");
    fs::write(&linked_file, "two").unwrap();
    let second = capture_run(&linked, "codex", "session_id", "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
}

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
    assert_success(&harness.run("codex", br#"{"session_id":"one"}"#));
    let first = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "one")
        .unwrap();
    fs::write(skills.join("skill-300"), "changed").unwrap();
    assert_success(&harness.run("codex", br#"{"session_id":"two"}"#));
    let second = harness
        .runs()
        .into_iter()
        .map(|path| read_json(path.join("run.json")))
        .find(|run| run["session_id"] == "two")
        .unwrap();

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
