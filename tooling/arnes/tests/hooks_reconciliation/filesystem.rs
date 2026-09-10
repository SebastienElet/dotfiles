use super::*;
#[test]
fn symlink_directory_and_unreadable_configurations_are_rejected_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let config = harness.config("codex")?;
    fs::create_dir_all(config.parent().ok_or("fixture path has no parent")?)?;
    let victim = harness.home.join("victim");
    fs::write(&victim, br#"{"hooks":{}}"#)?;
    symlink(&victim, &config)?;
    assert_failure(&harness.install("codex")?);
    assert_eq!(fs::read(&victim)?, br#"{"hooks":{}}"#);
    assert!(fs::symlink_metadata(&config)?.file_type().is_symlink());
    fs::remove_file(&config)?;
    fs::create_dir(&config)?;
    assert_failure(&harness.install("codex")?);
    assert!(config.is_dir());
    fs::remove_dir(&config)?;
    fs::write(&config, br#"{"hooks":{}}"#)?;
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000))?;
    assert_failure(&harness.install("codex")?);
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600))?;
    assert_eq!(fs::read(&config)?, br#"{"hooks":{}}"#);
    Ok(())
}
#[test]
fn fifo_hardlink_and_symlinked_agent_directory_are_rejected()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let config = harness.config("codex")?;
    fs::create_dir_all(config.parent().ok_or("fixture path has no parent")?)?;
    assert!(Command::new("mkfifo").arg(&config).status()?.success());
    assert_failure(&harness.install("codex")?);
    assert!(fs::symlink_metadata(&config)?.file_type().is_fifo());
    fs::remove_file(&config)?;
    let other = harness.home.join("other");
    fs::write(&other, br#"{"hooks":{}}"#)?;
    fs::hard_link(&other, &config)?;
    assert_failure(&harness.install("codex")?);
    assert_eq!(fs::read(&other)?, br#"{"hooks":{}}"#);
    fs::remove_file(&config)?;
    fs::remove_file(
        config
            .parent()
            .ok_or("fixture path has no parent")?
            .join(".hooks.json.lock"),
    )?;
    fs::remove_dir(config.parent().ok_or("fixture path has no parent")?)?;
    let actual = harness.home.join("actual-codex");
    fs::create_dir(&actual)?;
    symlink(
        &actual,
        config.parent().ok_or("fixture path has no parent")?,
    )?;
    assert_failure(&harness.install("codex")?);
    assert!(!actual.join("hooks.json").exists());
    Ok(())
}
#[test]
fn predictable_temporary_and_lock_symlinks_are_never_followed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let config = harness.config("codex")?;
    fs::create_dir_all(config.parent().ok_or("fixture path has no parent")?)?;
    let victim = harness.home.join("victim");
    fs::write(&victim, b"keep")?;
    let predictable = config
        .parent()
        .ok_or("fixture path has no parent")?
        .join("hooks.json.tmp");
    symlink(&victim, &predictable)?;
    assert_success(&harness.install("codex")?);
    assert_eq!(fs::read(&victim)?, b"keep");
    assert!(fs::symlink_metadata(&predictable)?.file_type().is_symlink());
    fs::remove_file(
        config
            .parent()
            .ok_or("fixture path has no parent")?
            .join(".hooks.json.lock"),
    )?;
    symlink(
        &victim,
        config
            .parent()
            .ok_or("fixture path has no parent")?
            .join(".hooks.json.lock"),
    )?;
    let before = fs::read(&config)?;
    assert_failure(&harness.install("codex")?);
    assert_eq!(fs::read(config)?, before);
    assert_eq!(fs::read(victim)?, b"keep");
    Ok(())
}
#[test]
fn creates_private_configuration_and_preserves_an_existing_mode()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    assert_success(&harness.install("codex")?);
    let config = harness.config("codex")?;
    assert_eq!(fs::metadata(&config)?.permissions().mode() & 0o777, 0o600);
    fs::set_permissions(&config, fs::Permissions::from_mode(0o640))?;
    let value = read_json(&config)?;
    let mut compact = serde_json::to_vec(&value)?;
    compact.push(b' ');
    fs::write(&config, compact)?;
    assert_success(&harness.install("codex")?);
    assert_eq!(fs::metadata(config)?.permissions().mode() & 0o777, 0o640);
    Ok(())
}
#[test]
fn unwritable_configuration_directory_is_rejected_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let config = harness.config("codex")?;
    let directory = config.parent().ok_or("fixture path has no parent")?;
    fs::create_dir_all(directory)?;
    fs::write(&config, br#"{"hooks":{}}"#)?;
    fs::set_permissions(directory, fs::Permissions::from_mode(0o500))?;
    let output = harness.install("codex")?;
    fs::set_permissions(directory, fs::Permissions::from_mode(0o700))?;
    assert_failure(&output);
    assert_eq!(fs::read(&config)?, br#"{"hooks":{}}"#);
    Ok(())
}
