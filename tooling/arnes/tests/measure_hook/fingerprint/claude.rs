use super::super::support::*;

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
