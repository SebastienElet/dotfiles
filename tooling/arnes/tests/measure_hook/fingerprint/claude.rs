use super::super::support::*;
#[test]
fn claude_fingerprint_hashes_only_enabled_registered_plugins()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0/plugin.json");
    let disabled = plugins.join("cache/marketplace/disabled/1.0.0/plugin.json");
    fs::create_dir_all(active.parent().ok_or("fixture path has no parent")?)?;
    fs::create_dir_all(disabled.parent().ok_or("fixture path has no parent")?)?;
    fs::write(&active, "active one")?;
    fs::write(&disabled, "disabled one")?;
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true,"disabled@marketplace":false}}"#,
    )?;
    fs :: write (plugins . join ("installed_plugins.json") , json ! ({ "version" : 2 , "plugins" : { "active@marketplace" : [{ "scope" : "user" , "installPath" : active .parent().ok_or("fixture path has no parent")? , "version" : "1.0.0" }] , "disabled@marketplace" : [{ "scope" : "user" , "installPath" : disabled .parent().ok_or("fixture path has no parent")? , "version" : "1.0.0" }] } }) . to_string () ,) ? ;
    let first = capture_run(&harness, "claude-code", "session_id", "one")?;
    fs::write(&disabled, "disabled two")?;
    let second = capture_run(&harness, "claude-code", "session_id", "two")?;
    assert_eq!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    fs::write(&active, "active two")?;
    let third = capture_run(&harness, "claude-code", "session_id", "three")?;
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
fn claude_fingerprint_does_not_follow_active_plugin_symlinks_outside_the_installation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active)?;
    fs::write(active.join("plugin.json"), "plugin")?;
    let transcript = harness.home.join("session.jsonl");
    fs::write(&transcript, "session one")?;
    symlink(&transcript, active.join("session.jsonl"))?;
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )?;
    fs :: write (plugins . join ("installed_plugins.json") , json ! ({ "version" : 2 , "plugins" : { "active@marketplace" : [{ "scope" : "user" , "installPath" : active , "version" : "1.0.0" }] } }) . to_string () ,) ? ;
    let first = capture_run(&harness, "claude-code", "session_id", "one")?;
    fs::write(transcript, "session two")?;
    let second = capture_run(&harness, "claude-code", "session_id", "two")?;
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
#[test]
fn claude_fingerprint_windows_oversized_active_plugin_files()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active)?;
    let plugin = active.join("plugin.bin");
    fs::write(&plugin, vec![b'x'; 1_048_577])?;
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )?;
    fs :: write (plugins . join ("installed_plugins.json") , json ! ({ "version" : 2 , "plugins" : { "active@marketplace" : [{ "scope" : "user" , "installPath" : active , "version" : "1.0.0" }] } }) . to_string () ,) ? ;
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
                    .contains("window"))
            }
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(std::convert::identity)
    );
    let mut file = fs::OpenOptions::new().write(true).open(plugin)?;
    file.seek(SeekFrom::End(-1))?;
    file.write_all(b"y")?;
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
#[test]
fn unsupported_claude_registry_excludes_plugin_files_with_a_limitation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let plugins = harness.home.join(".claude/plugins");
    let active = plugins.join("cache/marketplace/active/1.0.0");
    fs::create_dir_all(&active)?;
    fs::write(active.join("plugin.json"), "plugin one")?;
    fs::write(
        harness.home.join(".claude/settings.json"),
        r#"{"enabledPlugins":{"active@marketplace":true}}"#,
    )?;
    fs :: write (plugins . join ("installed_plugins.json") , json ! ({ "version" : 3 , "plugins" : { "active@marketplace" : [{ "scope" : "user" , "installPath" : active , "version" : "1.0.0" }] } }) . to_string () ,) ? ;
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
                    .contains("registry"))
            }
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .any(std::convert::identity)
    );
    fs::write(active.join("plugin.json"), "plugin two")?;
    let second = capture_run(&harness, "claude-code", "session_id", "two")?;
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
