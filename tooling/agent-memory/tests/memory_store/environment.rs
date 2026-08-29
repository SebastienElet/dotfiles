use super::support::*;

#[test]
fn memory_root_environment_worker() {
    let Some(output) = std::env::var_os(ROOT_WORKER_OUTPUT) else {
        return;
    };
    let result = MemoryRoot::from_environment();
    let value = match result {
        Ok(root) => root.path().to_string_lossy().into_owned(),
        Err(error) => format!("error:{}", error.code()),
    };
    fs::write(output, value).unwrap();
}

#[test]
fn resolves_the_override_or_default_memory_root_and_rejects_invalid_environment_paths() {
    let fixture = tempfile::tempdir().unwrap();
    let canonical_fixture = fs::canonicalize(fixture.path()).unwrap();
    let executable = std::env::current_exe().unwrap();
    let cases = [
        (
            "override",
            Some(fixture.path().join("override")),
            Some(fixture.path().join("home")),
            canonical_fixture.join("override").display().to_string(),
        ),
        (
            "default",
            None,
            Some(fixture.path().join("home")),
            canonical_fixture
                .join("home/.local/share/agent-memory")
                .display()
                .to_string(),
        ),
        (
            "relative override",
            Some(Path::new("relative").to_owned()),
            Some(fixture.path().join("home")),
            "error:unsafe_store_path".to_owned(),
        ),
        (
            "relative home",
            None,
            Some(Path::new("relative").to_owned()),
            "error:unsafe_store_path".to_owned(),
        ),
        (
            "missing home",
            None,
            None,
            "error:memory_root_unavailable".to_owned(),
        ),
    ];

    for (label, override_root, home, expected) in cases {
        let output = fixture.path().join(format!("{label}.txt"));
        let mut command = Command::new(&executable);
        command
            .arg("--exact")
            .arg("environment::memory_root_environment_worker")
            .arg("--nocapture")
            .env(ROOT_WORKER_OUTPUT, &output)
            .env_remove("ARNES_MEMORY_ROOT")
            .env_remove("HOME");
        if let Some(path) = override_root {
            command.env("ARNES_MEMORY_ROOT", path);
        }
        if let Some(path) = home {
            command.env("HOME", path);
        }

        let status = command.status().unwrap();

        assert!(status.success(), "{label}");
        assert_eq!(fs::read_to_string(output).unwrap(), expected, "{label}");
    }
}
