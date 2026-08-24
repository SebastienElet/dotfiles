use super::*;

#[test]
fn symlink_directory_and_unreadable_configurations_are_rejected_without_mutation() {
    let harness = Harness::new();
    let config = harness.config("codex");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let victim = harness.home.join("victim");
    fs::write(&victim, br#"{"hooks":{}}"#).unwrap();
    symlink(&victim, &config).unwrap();

    assert_failure(&harness.install("codex"));
    assert_eq!(fs::read(&victim).unwrap(), br#"{"hooks":{}}"#);
    assert!(
        fs::symlink_metadata(&config)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    fs::remove_file(&config).unwrap();
    fs::create_dir(&config).unwrap();
    assert_failure(&harness.install("codex"));
    assert!(config.is_dir());

    fs::remove_dir(&config).unwrap();
    fs::write(&config, br#"{"hooks":{}}"#).unwrap();
    fs::set_permissions(&config, fs::Permissions::from_mode(0o000)).unwrap();
    assert_failure(&harness.install("codex"));
    fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
    assert_eq!(fs::read(&config).unwrap(), br#"{"hooks":{}}"#);
}

#[test]
fn fifo_hardlink_and_symlinked_agent_directory_are_rejected() {
    let harness = Harness::new();
    let config = harness.config("codex");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    assert!(
        Command::new("mkfifo")
            .arg(&config)
            .status()
            .unwrap()
            .success()
    );
    assert_failure(&harness.install("codex"));
    assert!(fs::symlink_metadata(&config).unwrap().file_type().is_fifo());

    fs::remove_file(&config).unwrap();
    let other = harness.home.join("other");
    fs::write(&other, br#"{"hooks":{}}"#).unwrap();
    fs::hard_link(&other, &config).unwrap();
    assert_failure(&harness.install("codex"));
    assert_eq!(fs::read(&other).unwrap(), br#"{"hooks":{}}"#);

    fs::remove_file(&config).unwrap();
    fs::remove_file(config.parent().unwrap().join(".hooks.json.lock")).unwrap();
    fs::remove_dir(config.parent().unwrap()).unwrap();
    let actual = harness.home.join("actual-codex");
    fs::create_dir(&actual).unwrap();
    symlink(&actual, config.parent().unwrap()).unwrap();
    assert_failure(&harness.install("codex"));
    assert!(!actual.join("hooks.json").exists());
}

#[test]
fn predictable_temporary_and_lock_symlinks_are_never_followed() {
    let harness = Harness::new();
    let config = harness.config("codex");
    fs::create_dir_all(config.parent().unwrap()).unwrap();
    let victim = harness.home.join("victim");
    fs::write(&victim, b"keep").unwrap();
    let predictable = config.parent().unwrap().join("hooks.json.tmp");
    symlink(&victim, &predictable).unwrap();

    assert_success(&harness.install("codex"));
    assert_eq!(fs::read(&victim).unwrap(), b"keep");
    assert!(
        fs::symlink_metadata(&predictable)
            .unwrap()
            .file_type()
            .is_symlink()
    );

    fs::remove_file(config.parent().unwrap().join(".hooks.json.lock")).unwrap();
    symlink(&victim, config.parent().unwrap().join(".hooks.json.lock")).unwrap();
    let before = fs::read(&config).unwrap();
    assert_failure(&harness.install("codex"));
    assert_eq!(fs::read(config).unwrap(), before);
    assert_eq!(fs::read(victim).unwrap(), b"keep");
}

#[test]
fn creates_private_configuration_and_preserves_an_existing_mode() {
    let harness = Harness::new();
    assert_success(&harness.install("codex"));
    let config = harness.config("codex");
    assert_eq!(
        fs::metadata(&config).unwrap().permissions().mode() & 0o777,
        0o600
    );

    fs::set_permissions(&config, fs::Permissions::from_mode(0o640)).unwrap();
    let value = read_json(&config);
    let mut compact = serde_json::to_vec(&value).unwrap();
    compact.push(b' ');
    fs::write(&config, compact).unwrap();
    assert_success(&harness.install("codex"));
    assert_eq!(
        fs::metadata(config).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn unwritable_configuration_directory_is_rejected_without_mutation() {
    let harness = Harness::new();
    let config = harness.config("codex");
    let directory = config.parent().unwrap();
    fs::create_dir_all(directory).unwrap();
    fs::write(&config, br#"{"hooks":{}}"#).unwrap();
    fs::set_permissions(directory, fs::Permissions::from_mode(0o500)).unwrap();

    let output = harness.install("codex");

    fs::set_permissions(directory, fs::Permissions::from_mode(0o700)).unwrap();
    assert_failure(&output);
    assert_eq!(fs::read(&config).unwrap(), br#"{"hooks":{}}"#);
}
