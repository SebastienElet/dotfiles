use super::support::*;
#[test]
fn refuses_relative_state_and_state_inside_the_observed_repository()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let payload = br#"{"session_id":"session","event":"SessionStart"}"#;
    let mut relative = harness.command("codex");
    relative.env("XDG_STATE_HOME", "relative/state");
    let mut child = relative.spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(payload)?;
    let output = child.wait_with_output()?;
    assert_advisory_failure(&output);
    assert!(String::from_utf8(output.stderr)?.contains("absolute"));
    git(&harness.repository, &["init"])?;
    let mut inside = harness.command("codex");
    inside.env("XDG_STATE_HOME", harness.repository.join("state"));
    let mut child = inside.spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(payload)?;
    let output = child.wait_with_output()?;
    assert_advisory_failure(&output);
    assert!(String::from_utf8(output.stderr)?.contains("repository"));
    Ok(())
}
#[test]
fn refuses_state_inside_git_root_when_git_is_unavailable_from_a_subdirectory()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    git(&harness.repository, &["init"])?;
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested)?;
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
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(br#"{"session_id":"session"}"#)?;
    let output = child.wait_with_output()?;
    assert_advisory_failure(&output);
    assert!(!harness.repository.join("state").exists());
    Ok(())
}
#[test]
fn refuses_state_inside_repository_observed_only_through_git_environment()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    let git_dir = harness.root.path().join("external.git");
    git(
        &harness.repository,
        &[
            "init",
            "--bare",
            git_dir.to_str().ok_or("required test value is missing")?,
        ],
    )?;
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested)?;
    let state = harness.repository.join("state");
    let mut command = harness.command("codex");
    command
        .current_dir(nested)
        .env("GIT_DIR", &git_dir)
        .env("GIT_WORK_TREE", &harness.repository)
        .env("XDG_STATE_HOME", &state);
    let mut child = command.spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(br#"{"session_id":"session"}"#)?;
    let output = child.wait_with_output()?;
    assert_advisory_failure(&output);
    assert!(!state.exists());
    Ok(())
}
#[test]
fn nested_fake_git_marker_cannot_shrink_the_protected_repository()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    git(&harness.repository, &["init"])?;
    let nested = harness.repository.join("nested");
    fs::create_dir(&nested)?;
    fs::create_dir(nested.join(".git"))?;
    let current = nested.join("deeper");
    fs::create_dir(&current)?;
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
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(br#"{"session_id":"session"}"#)?;
    let output = child.wait_with_output()?;
    assert_advisory_failure(&output);
    assert!(!harness.repository.join("state").exists());
    Ok(())
}
#[test]
fn nested_git_repository_is_observed_while_both_repository_boundaries_are_protected()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let (harness, inner) = nested_repositories()?;
    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"one"}"#,
    )?);
    let first = run_record(&harness, "one")?;
    assert!(first.get("repository").is_none());
    assert_eq!(
        *(first)
            .get("repository_commit")
            .ok_or("missing fixture index repository_commit")?,
        git_value(&inner, &["rev-parse", "HEAD"])?
    );
    assert!(first.get("repository_branch").is_none());
    fs::write(harness.repository.join("AGENTS.md"), "outer two")?;
    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"two"}"#,
    )?);
    let second = run_record(&harness, "two")?;
    assert_eq!(
        *(first)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    fs::write(inner.join("AGENTS.md"), "inner two")?;
    assert_success(&run_at(
        &harness,
        &inner,
        &harness.state,
        br#"{"session_id":"three"}"#,
    )?);
    let third = run_record(&harness, "three")?;
    assert_ne!(
        *(second)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?,
        *(third)
            .get("harness_fingerprint")
            .ok_or("missing fixture index harness_fingerprint")?
    );
    let _: () = for state in [inner.join("state"), harness.repository.join("state")] {
        let output = run_at(&harness, &inner, &state, br#"{"session_id":"blocked"}"#)?;
        assert_advisory_failure(&output);
        assert!(!state.exists());
    };
    Ok(())
}
fn nested_repositories() -> Result<(Harness, PathBuf), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    init_repository(&harness.repository, "outer", "outer-file")?;
    fs::write(harness.repository.join("AGENTS.md"), "outer one")?;
    let inner = harness.repository.join("inner");
    fs::create_dir(&inner)?;
    init_repository(&inner, "inner", "inner-file")?;
    fs::write(inner.join("AGENTS.md"), "inner one")?;
    Ok((harness, inner))
}
#[test]
fn git_repository_path_with_trailing_spaces_is_not_persisted()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::with_repository_name("repository ")?;
    git(&harness.repository, &["init", "-b", "measurement"])?;
    fs::write(harness.repository.join("tracked"), "tracked")?;
    git(&harness.repository, &["add", "tracked"])?;
    commit(&harness.repository, "initial")?;
    fs::write(harness.repository.join("AGENTS.md"), "one")?;
    let first = capture_run(&harness, "codex", "session_id", "one")?;
    assert!(first.get("repository").is_none());
    fs::write(harness.repository.join("AGENTS.md"), "two")?;
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
