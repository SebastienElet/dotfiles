use super::super::support::*;

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
