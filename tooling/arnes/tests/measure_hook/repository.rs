use super::support::*;

#[test]
fn refuses_relative_state_and_state_inside_the_observed_repository() {
    let harness = Harness::new();
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    let mut relative = harness.command("codex");
    relative.env("XDG_STATE_HOME", "relative/state");
    let mut child = relative.spawn().unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_advisory_failure(&output);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("absolute")
    );

    git(&harness.repository, &["init"]);
    let mut inside = harness.command("codex");
    inside.env("XDG_STATE_HOME", harness.repository.join("state"));
    let mut child = inside.spawn().unwrap();
    child.stdin.take().unwrap().write_all(payload).unwrap();
    let output = child.wait_with_output().unwrap();
    assert_advisory_failure(&output);
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("repository")
    );
}

#[test]
fn refuses_state_inside_git_root_when_git_is_unavailable_from_a_subdirectory() {
    let harness = Harness::new();
    git(&harness.repository, &["init"]);
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["measure", "hook", "--agent", "codex"])
        .current_dir(nested)
        .env_clear()
        .env("HOME", &harness.home)
        .env("PATH", "/nonexistent")
        .env("XDG_STATE_HOME", harness.repository.join("state"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session"}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_advisory_failure(&output);
    assert!(!harness.repository.join("state").exists());
}

#[test]
fn refuses_state_inside_repository_observed_only_through_git_environment() {
    let harness = Harness::new();
    let git_dir = harness._root.path().join("external.git");
    git(
        &harness.repository,
        &["init", "--bare", git_dir.to_str().unwrap()],
    );
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested).unwrap();
    let state = harness.repository.join("state");
    let mut command = harness.command("codex");
    command
        .current_dir(nested)
        .env("GIT_DIR", &git_dir)
        .env("GIT_WORK_TREE", &harness.repository)
        .env("XDG_STATE_HOME", &state);
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session"}"#)
        .unwrap();

    let output = child.wait_with_output().unwrap();

    assert_advisory_failure(&output);
    assert!(!state.exists());
}

#[test]
fn nested_fake_git_marker_cannot_shrink_the_protected_repository() {
    let harness = Harness::new();
    git(&harness.repository, &["init"]);
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested).unwrap();
    fs::create_dir(nested.join(".git")).unwrap();
    let current = nested.join("deeper");
    fs::create_dir(&current).unwrap();
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["measure", "hook", "--agent", "codex"])
        .current_dir(current)
        .env_clear()
        .env("HOME", &harness.home)
        .env("PATH", "/nonexistent")
        .env("XDG_STATE_HOME", harness.repository.join("state"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(br#"{"session_id":"session"}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();

    assert_advisory_failure(&output);
    assert!(!harness.repository.join("state").exists());
}

#[test]
fn nested_git_repository_is_observed_while_both_repository_boundaries_are_protected() {
    let (harness, inner) = nested_repositories();

    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"one"}"#,
    ));
    let first = run_record(&harness, "one");
    assert!(first.get("repository").is_none());
    assert_eq!(
        first["repository_commit"],
        git_value(&inner, &["rev-parse", "HEAD"])
    );
    assert!(first.get("repository_branch").is_none());

    fs::write(harness.repository.join("AGENTS.md"), "outer two").unwrap();
    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"two"}"#,
    ));
    let second = run_record(&harness, "two");
    assert_eq!(first["harness_fingerprint"], second["harness_fingerprint"]);
    fs::write(inner.join("AGENTS.md"), "inner two").unwrap();
    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"three"}"#,
    ));
    let third = run_record(&harness, "three");
    assert_ne!(second["harness_fingerprint"], third["harness_fingerprint"]);

    for state in [inner.join("state"), harness.repository.join("state")] {
        let output = run_at(&harness, &inner, &state, br#"{"session_id":"blocked"}"#);
        assert_advisory_failure(&output);
        assert!(!state.exists());
    }
}

fn nested_repositories() -> (Harness, PathBuf) {
    let harness = Harness::new();
    init_repository(&harness.repository, "outer", "outer-file");
    fs::write(harness.repository.join("AGENTS.md"), "outer one").unwrap();
    let inner = harness.repository.join("inner");
    fs::create_dir(&inner).unwrap();
    init_repository(&inner, "inner", "inner-file");
    fs::write(inner.join("AGENTS.md"), "inner one").unwrap();
    (harness, inner)
}

#[test]
fn git_repository_path_with_trailing_spaces_is_not_persisted() {
    let harness = Harness::with_repository_name("repository ");
    git(&harness.repository, &["init", "-b", "measurement"]);
    fs::write(harness.repository.join("tracked"), "tracked").unwrap();
    git(&harness.repository, &["add", "tracked"]);
    commit(&harness.repository, "initial");
    fs::write(harness.repository.join("AGENTS.md"), "one").unwrap();

    let first = capture_run(&harness, "codex", "session_id", "one");
    assert!(first.get("repository").is_none());
    fs::write(harness.repository.join("AGENTS.md"), "two").unwrap();
    let second = capture_run(&harness, "codex", "session_id", "two");
    assert_ne!(first["harness_fingerprint"], second["harness_fingerprint"]);
}
