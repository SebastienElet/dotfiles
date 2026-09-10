use super::super::support::*;
#[test]
fn codex_fingerprint_hashes_only_single_version_enabled_plugins()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    fs::create_dir_all(harness.home.join(".codex"))?;
    fs::write(
        harness.home.join(".codex/config.toml"),
        "[plugins.\"active@marketplace\"]\nenabled = true\n[plugins.\"disabled@marketplace\"]\nenabled = false\n",
    )?;
    let active = harness
        .home
        .join(".codex/plugins/cache/marketplace/active/1.0.0/plugin.json");
    let disabled = harness
        .home
        .join(".codex/plugins/cache/marketplace/disabled/1.0.0/plugin.json");
    fs::create_dir_all(active.parent().ok_or("fixture path has no parent")?)?;
    fs::create_dir_all(disabled.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&active, "active one")?;
    fs::write(&disabled, "disabled one")?;
    let first = capture_run(&harness, "codex", "session_id", "one")?;
    fs::write(&disabled, "disabled two")?;
    let second = capture_run(&harness, "codex", "session_id", "two")?;
    assert_eq!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    fs::write(&active, "active two")?;
    let third = capture_run(&harness, "codex", "session_id", "three")?;
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
fn codex_fingerprint_records_ambiguous_enabled_plugin_versions()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    fs::create_dir_all(harness.home.join(".codex"))?;
    fs::write(
        harness.home.join(".codex/config.toml"),
        "[plugins.\"demo@marketplace\"]\nenabled = true\n",
    )?;
    let cache = harness.home.join(".codex/plugins/cache/marketplace/demo");
    for version in ["1.0.0", "2.0.0"] {
        fs::create_dir_all(cache.join(version))?;
        fs::write(cache.join(version).join("plugin.json"), version)?;
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
                    .contains("ambiguous"))
            }
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(std::convert::identity)
    );
    fs::create_dir_all(cache.join("3.0.0"))?;
    fs::write(cache.join("3.0.0/plugin.json"), "three")?;
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
fn codex_ambiguous_version_sets_have_unambiguous_fingerprints()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let capture = |versions: &[&str]| -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let harness = Harness::new()?;
        fs::create_dir_all(harness.home.join(".codex"))?;
        fs::write(
            harness.home.join(".codex/config.toml"),
            "[plugins.\"demo@marketplace\"]\nenabled = true\n",
        )?;
        let cache = harness.home.join(".codex/plugins/cache/marketplace/demo");
        for version in versions {
            fs::create_dir_all(cache.join(version))?;
        }
        Ok((*(capture_run(&harness, "codex", "session_id", "session")?)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?)
        .as_str()
        .ok_or("expected a fingerprint string")?
        .to_owned())
    };
    assert_ne!(capture(&["a,b", "c"])?, capture(&["a", "b,c"])?);
    Ok(())
}
#[test]
fn codex_plugin_roots_cannot_escape_the_cache()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let escaped = Harness::new()?;
    fs::create_dir_all(escaped.home.join(".codex"))?;
    fs::write(
        escaped.home.join(".codex/config.toml"),
        "[plugins.\"payload@../../../outside\"]\nenabled = true\n",
    )?;
    let escaped_file = escaped.home.join("outside/payload/1.0.0/plugin.json");
    fs::create_dir_all(escaped_file.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&escaped_file, "one")?;
    let first = capture_run(&escaped, "codex", "session_id", "one")?;
    fs::write(&escaped_file, "two")?;
    let second = capture_run(&escaped, "codex", "session_id", "two")?;
    assert_eq!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    let linked = Harness::new()?;
    fs::create_dir_all(linked.home.join(".codex/plugins/cache/marketplace"))?;
    fs::write(
        linked.home.join(".codex/config.toml"),
        "[plugins.\"active@marketplace\"]\nenabled = true\n",
    )?;
    let outside = linked.home.join("outside/active");
    let linked_file = outside.join("1.0.0/plugin.json");
    fs::create_dir_all(linked_file.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&linked_file, "one")?;
    symlink(
        &outside,
        linked.home.join(".codex/plugins/cache/marketplace/active"),
    )?;
    let first = capture_run(&linked, "codex", "session_id", "one")?;
    fs::write(&linked_file, "two")?;
    let second = capture_run(&linked, "codex", "session_id", "two")?;
    assert_eq!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    Ok(())
}
