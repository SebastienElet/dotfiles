use super::{ConfigFile, set_after_publish_hook};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};

#[test]
fn publication_identity_matches_the_created_file_and_conforming_replay()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let path = std::path::Path::new(".claude/commands/review.md");
    let identity = ConfigFile::open_path(root.path(), path)?.replace_with_identity(b"expected")?;
    let metadata = fs::metadata(root.path().join(path))?;
    assert_eq!(identity, (metadata.dev(), metadata.ino()));
    let replay = ConfigFile::open_path(root.path(), path)?.replace_with_identity(b"expected")?;
    assert_eq!(replay, identity);
    let changed = ConfigFile::open_path(root.path(), path)?.replace_with_identity(b"updated")?;
    let metadata = fs::metadata(root.path().join(path))?;
    assert_eq!(changed, (metadata.dev(), metadata.ino()));
    assert_ne!(changed, identity);
    Ok(())
}

#[test]
fn byte_identical_replacement_after_publication_cannot_obtain_an_owned_identity()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let directory = root.path().join(".claude");
    let path = directory.join("review.md");
    let newer = directory.join("newer.md");
    let config = ConfigFile::open(root.path(), ".claude", "review.md")?;
    let path_for_hook = path.clone();
    set_after_publish_hook(move || {
        assert!(fs::write(&newer, b"expected").is_ok());
        assert!(fs::rename(&newer, &path_for_hook).is_ok());
    });
    assert!(config.replace_with_identity(b"expected").is_err());
    assert_eq!(fs::read(path)?, b"expected");
    Ok(())
}

#[test]
fn nested_paths_create_regular_files_and_refuse_linked_ancestors()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let foreign = root.path().join("foreign");
    fs::create_dir_all(&home)?;
    fs::create_dir_all(&foreign)?;
    let path = std::path::Path::new(".claude/commands/review.md");
    ConfigFile::open_path(&home, path)?.replace(b"expected")?;
    assert_eq!(fs::read(home.join(path))?, b"expected");
    fs::remove_dir_all(home.join(".claude"))?;
    symlink(&foreign, home.join(".claude"))?;
    assert!(ConfigFile::open_path(&home, path).is_err());
    assert!(!foreign.join("commands").exists());
    Ok(())
}

#[test]
fn nested_path_publication_refuses_relocated_ancestors() -> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".claude/commands");
    let moved = root.path().join("foreign");
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("review.md"), b"original")?;
    let config = ConfigFile::open_path(&home, std::path::Path::new(".claude/commands/review.md"))?;
    fs::rename(home.join(".claude"), &moved)?;
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("review.md"), b"concurrent")?;
    assert!(config.replace(b"replacement").is_err());
    assert_eq!(fs::read(moved.join("commands/review.md"))?, b"original");
    assert_eq!(fs::read(directory.join("review.md"))?, b"concurrent");
    Ok(())
}

#[test]
fn unconfined_nested_paths_are_refused_before_creating_directories()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    fs::create_dir(&home)?;
    for invalid in [".claude/../foreign.md", "/absolute.md", "", ".."] {
        assert!(ConfigFile::open_path(&home, std::path::Path::new(invalid)).is_err());
    }
    assert_eq!(fs::read_dir(&home)?.count(), 0);
    Ok(())
}

#[test]
fn refuses_publication_when_the_configuration_directory_moves_into_a_source()
-> Result<(), Box<dyn std::error::Error>> {
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let moved = root.path().join("source");
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("config.toml"), b"original")?;
    let config = ConfigFile::open(&home, ".codex", "config.toml")?;
    fs::rename(&directory, &moved)?;
    fs::create_dir(&directory)?;
    fs::write(directory.join("config.toml"), b"concurrent")?;
    assert!(config.replace(b"replacement").is_err());
    assert_eq!(fs::read(moved.join("config.toml"))?, b"original");
    assert_eq!(fs::read(directory.join("config.toml"))?, b"concurrent");
    Ok(())
}

#[test]
fn refuses_publication_when_the_scope_root_is_relocated() -> Result<(), Box<dyn std::error::Error>>
{
    let root = tempfile::tempdir()?;
    let home = root.path().join("home");
    let directory = home.join(".codex");
    let moved = root.path().join("source");
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("config.toml"), b"original")?;
    let config = ConfigFile::open(&home, ".codex", "config.toml")?;
    fs::rename(&home, &moved)?;
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("config.toml"), b"concurrent")?;
    assert!(config.replace(b"replacement").is_err());
    assert_eq!(fs::read(moved.join(".codex/config.toml"))?, b"original");
    assert_eq!(fs::read(directory.join("config.toml"))?, b"concurrent");
    Ok(())
}

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
