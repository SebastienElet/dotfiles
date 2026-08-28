use super::support::*;

#[test]
fn creates_the_private_store_layout() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");

    Store::open(memory_root(&root)).unwrap();

    for directory in [
        root.clone(),
        root.join("entries"),
        root.join("entries/user"),
        root.join("entries/project"),
    ] {
        assert!(directory.is_dir(), "{}", directory.display());
        assert_eq!(private_mode(&directory), 0o700, "{}", directory.display());
    }
    for file in [
        root.join(".lock"),
        root.join("index.json"),
        root.join("oracle-cache.json"),
    ] {
        assert!(file.is_file(), "{}", file.display());
        assert_eq!(private_mode(&file), 0o600, "{}", file.display());
    }
}

#[test]
fn repairs_open_store_permissions_on_the_opened_objects() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    fs::create_dir_all(root.join("entries/user")).unwrap();
    fs::create_dir_all(root.join("entries/project")).unwrap();
    fs::write(root.join(".lock"), b"").unwrap();
    fs::write(root.join("index.json"), b"{}\n").unwrap();
    fs::write(root.join("oracle-cache.json"), b"{}\n").unwrap();
    for directory in [
        root.clone(),
        root.join("entries"),
        root.join("entries/user"),
        root.join("entries/project"),
    ] {
        fs::set_permissions(&directory, fs::Permissions::from_mode(0o777)).unwrap();
    }
    for file in [
        root.join(".lock"),
        root.join("index.json"),
        root.join("oracle-cache.json"),
    ] {
        fs::set_permissions(&file, fs::Permissions::from_mode(0o666)).unwrap();
    }

    Store::open(memory_root(&root)).unwrap();

    for directory in [
        root.clone(),
        root.join("entries"),
        root.join("entries/user"),
        root.join("entries/project"),
    ] {
        assert_eq!(private_mode(&directory), 0o700, "{}", directory.display());
    }
    for file in [
        root.join(".lock"),
        root.join("index.json"),
        root.join("oracle-cache.json"),
    ] {
        assert_eq!(private_mode(&file), 0o600, "{}", file.display());
    }
}

#[test]
fn refuses_a_mode_repair_failure_instead_of_opening_an_unprotected_store() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    fs::create_dir(&root).unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755)).unwrap();

    let error = Store::open_with_failpoint(memory_root(&root), StoreFailpoint::BeforeModeRepair)
        .unwrap_err();

    assert_eq!(error.code(), "store_permissions_unavailable");
    assert_eq!(private_mode(&root), 0o755);
    assert!(!root.join(".lock").exists());
}

#[test]
fn refuses_root_and_controlled_component_symlinks_without_touching_targets() {
    let cases = [
        "root",
        "entries",
        "entries/user",
        "entries/project",
        ".lock",
        "index.json",
        "oracle-cache.json",
    ];

    for controlled_component in cases {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let target = fixture.path().join("outside");
        fs::create_dir(&target).unwrap();
        fs::write(target.join("sentinel"), b"unchanged").unwrap();
        if controlled_component == "root" {
            symlink(&target, &root).unwrap();
        } else {
            fs::create_dir(&root).unwrap();
            let controlled = root.join(controlled_component);
            if controlled_component.contains('/') {
                fs::create_dir_all(controlled.parent().unwrap()).unwrap();
            }
            symlink(&target, controlled).unwrap();
        }

        let error = Store::open(memory_root(&root)).unwrap_err();

        assert_eq!(error.code(), "unsafe_store_path", "{controlled_component}");
        assert_eq!(fs::read(target.join("sentinel")).unwrap(), b"unchanged");
        assert_eq!(fs::read_dir(&target).unwrap().count(), 1);
    }
}

#[test]
fn refuses_a_non_directory_parent_and_non_absolute_root() {
    let fixture = tempfile::tempdir().unwrap();
    let parent = fixture.path().join("not-a-directory");
    fs::write(&parent, b"unchanged").unwrap();

    let error = Store::open(memory_root(&parent.join("agent-memory"))).unwrap_err();
    let relative_error = MemoryRoot::new(Path::new("relative/agent-memory")).unwrap_err();

    assert_eq!(error.code(), "unsafe_store_path");
    assert_eq!(relative_error.code(), "unsafe_store_path");
    assert_eq!(fs::read(parent).unwrap(), b"unchanged");
}

#[test]
fn refuses_hardlinked_managed_files_without_repairing_the_external_inode() {
    for managed_name in [".lock", "index.json", "oracle-cache.json"] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let outside = fixture.path().join("outside");
        fs::create_dir(&root).unwrap();
        fs::write(&outside, b"unchanged").unwrap();
        fs::set_permissions(&outside, fs::Permissions::from_mode(0o644)).unwrap();
        fs::hard_link(&outside, root.join(managed_name)).unwrap();

        let error = Store::open(memory_root(&root)).unwrap_err();

        assert_eq!(error.code(), "unsafe_store_path", "{managed_name}");
        assert_eq!(fs::read(&outside).unwrap(), b"unchanged");
        assert_eq!(private_mode(&outside), 0o644);
    }
}

#[test]
fn refuses_id_traversal_before_reading_any_neighbor() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let neighbor = fixture.path().join("outside.yaml");
    fs::write(&neighbor, b"private neighbor").unwrap();
    let store = Store::open(memory_root(&root)).unwrap();

    let error = store.load("../outside").unwrap_err();

    assert_eq!(error.code(), "invalid_memory_id");
    assert_eq!(fs::read(neighbor).unwrap(), b"private neighbor");
}
