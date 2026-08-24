use super::{ConfigFile, set_after_publish_hook};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn refuses_an_in_place_mutation_after_validation_and_restores_the_newer_content() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".codex", "hooks.json").unwrap();

    fs::write(&path, b"newer").unwrap();
    let error = config.replace(b"replacement").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path).unwrap(), b"newer");
}

#[test]
fn refuses_an_atomic_replacement_after_validation_and_restores_the_newer_file() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".cursor");
    let path = directory.join("hooks.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".cursor", "hooks.json").unwrap();

    fs::write(&newer, b"newer").unwrap();
    fs::rename(&newer, &path).unwrap();
    let error = config.replace(b"replacement").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path).unwrap(), b"newer");
}

#[test]
fn refuses_idempotent_installation_after_configuration_is_deleted() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".codex", "hooks.json").unwrap();

    fs::remove_file(&path).unwrap();
    let error = config.replace(b"original").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert!(!path.exists());
}

#[test]
fn refuses_idempotent_installation_after_identical_atomic_replacement() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".claude");
    let path = directory.join("settings.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".claude", "settings.json").unwrap();

    fs::write(&newer, b"original").unwrap();
    fs::rename(&newer, &path).unwrap();
    let error = config.replace(b"original").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path).unwrap(), b"original");
}

#[test]
fn refuses_idempotent_installation_after_configuration_becomes_a_symlink() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".cursor");
    let path = directory.join("hooks.json");
    let victim = home.join("victim.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".cursor", "hooks.json").unwrap();

    fs::write(&victim, b"original").unwrap();
    fs::remove_file(&path).unwrap();
    symlink(&victim, &path).unwrap();
    assert!(config.replace(b"original").is_err());

    assert!(fs::symlink_metadata(path).unwrap().file_type().is_symlink());
    assert_eq!(fs::read(victim).unwrap(), b"original");
}

#[test]
fn refuses_a_replacement_after_publish_without_overwriting_the_newer_value() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".codex", "hooks.json").unwrap();

    let path_for_hook = path.clone();
    set_after_publish_hook(move || {
        fs::write(&newer, b"newer").unwrap();
        fs::rename(&newer, &path_for_hook).unwrap();
    });
    let error = config.replace(b"replacement").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path).unwrap(), b"newer");
}

#[test]
fn refuses_an_in_place_mutation_after_publish_without_restoring_old_bytes() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".claude");
    let path = directory.join("settings.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".claude", "settings.json").unwrap();

    let path_for_hook = path.clone();
    set_after_publish_hook(move || fs::write(path_for_hook, b"newer").unwrap());
    let error = config.replace(b"replacement").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path).unwrap(), b"newer");
}

#[test]
fn refuses_a_mode_mutation_after_publish_without_resetting_the_new_mode() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".cursor");
    let path = directory.join("hooks.json");
    fs::create_dir_all(&directory).unwrap();
    fs::write(&path, b"original").unwrap();
    let config = ConfigFile::open(&home, ".cursor", "hooks.json").unwrap();

    let path_for_hook = path.clone();
    set_after_publish_hook(move || {
        fs::set_permissions(path_for_hook, fs::Permissions::from_mode(0o640)).unwrap();
    });
    let error = config.replace(b"replacement").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(
        fs::metadata(path).unwrap().permissions().mode() & 0o777,
        0o640
    );
}

#[test]
fn refuses_a_replacement_after_first_publish_without_overwriting_it() {
    let root = tempfile::tempdir().unwrap();
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory).unwrap();
    let config = ConfigFile::open(&home, ".codex", "hooks.json").unwrap();

    let path_for_hook = path.clone();
    set_after_publish_hook(move || {
        fs::write(&newer, b"newer").unwrap();
        fs::rename(&newer, &path_for_hook).unwrap();
    });
    let error = config.replace(b"replacement").unwrap_err();

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path).unwrap(), b"newer");
}
