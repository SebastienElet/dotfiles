use super::{ConfigFile, set_after_publish_hook};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};

#[test]
fn refuses_an_in_place_mutation_after_validation_and_restores_the_newer_content()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".codex", "hooks.json")?;

    fs::write(&path, b"newer")?;
    let error = config
        .replace(b"replacement")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path)?, b"newer");
    Ok(())
}

#[test]
fn refuses_an_atomic_replacement_after_validation_and_restores_the_newer_file()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".cursor");
    let path = directory.join("hooks.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".cursor", "hooks.json")?;

    fs::write(&newer, b"newer")?;
    fs::rename(&newer, &path)?;
    let error = config
        .replace(b"replacement")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path)?, b"newer");
    Ok(())
}

#[test]
fn refuses_idempotent_installation_after_configuration_is_deleted()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".codex", "hooks.json")?;

    fs::remove_file(&path)?;
    let error = config
        .replace(b"original")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert!(!path.exists());
    Ok(())
}

#[test]
fn refuses_idempotent_installation_after_identical_atomic_replacement()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".claude");
    let path = directory.join("settings.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".claude", "settings.json")?;

    fs::write(&newer, b"original")?;
    fs::rename(&newer, &path)?;
    let error = config
        .replace(b"original")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path)?, b"original");
    Ok(())
}

#[test]
fn refuses_idempotent_installation_after_configuration_becomes_a_symlink()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".cursor");
    let path = directory.join("hooks.json");
    let victim = home.join("victim.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".cursor", "hooks.json")?;

    fs::write(&victim, b"original")?;
    fs::remove_file(&path)?;
    symlink(&victim, &path)?;
    assert!(config.replace(b"original").is_err());

    assert!(fs::symlink_metadata(path)?.file_type().is_symlink());
    assert_eq!(fs::read(victim)?, b"original");
    Ok(())
}

#[test]
fn refuses_a_replacement_after_publish_without_overwriting_the_newer_value()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".codex", "hooks.json")?;

    let path_for_hook = path.clone();
    set_after_publish_hook(move || {
        assert!(fs::write(&newer, b"newer").is_ok());
        assert!(fs::rename(&newer, &path_for_hook).is_ok());
    });
    let error = config
        .replace(b"replacement")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path)?, b"newer");
    Ok(())
}

#[test]
fn refuses_an_in_place_mutation_after_publish_without_restoring_old_bytes()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".claude");
    let path = directory.join("settings.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".claude", "settings.json")?;

    let path_for_hook = path.clone();
    set_after_publish_hook(move || assert!(fs::write(path_for_hook, b"newer").is_ok()));
    let error = config
        .replace(b"replacement")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path)?, b"newer");
    Ok(())
}

#[test]
fn refuses_a_mode_mutation_after_publish_without_resetting_the_new_mode()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".cursor");
    let path = directory.join("hooks.json");
    fs::create_dir_all(&directory)?;
    fs::write(&path, b"original")?;
    let config = ConfigFile::open(&home, ".cursor", "hooks.json")?;

    let path_for_hook = path.clone();
    set_after_publish_hook(move || {
        assert!(fs::set_permissions(path_for_hook, fs::Permissions::from_mode(0o640)).is_ok());
    });
    let error = config
        .replace(b"replacement")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::metadata(path)?.permissions().mode() & 0o777, 0o640);
    Ok(())
}

#[test]
fn refuses_a_replacement_after_first_publish_without_overwriting_it()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let path = directory.join("hooks.json");
    let newer = directory.join("newer.json");
    fs::create_dir_all(&directory)?;
    let config = ConfigFile::open(&home, ".codex", "hooks.json")?;

    let path_for_hook = path.clone();
    set_after_publish_hook(move || {
        assert!(fs::write(&newer, b"newer").is_ok());
        assert!(fs::rename(&newer, &path_for_hook).is_ok());
    });
    let error = config
        .replace(b"replacement")
        .err()
        .ok_or("expected operation to fail")?;

    assert!(error.to_string().contains("changed during installation"));
    assert_eq!(fs::read(path)?, b"newer");
    Ok(())
}
