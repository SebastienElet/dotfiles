#![cfg(test)]

use crate::memory_support::{FakeProcessRunner, FakeResponse, git};
use agent_memory::{MemoryErrorClass, ProcessRunner, SystemProcessRunner, resolve_project};
use std::ffi::{OsStr, OsString};
use std::fs;

#[test]
fn shares_project_identity_between_linked_worktrees_but_not_clones()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let root = tempfile::tempdir()?;
    let main = root.path().join("main");
    let linked = root.path().join("linked");
    let clone = root.path().join("clone");
    fs::create_dir(&main)?;
    git(&main, &["init"])?;
    fs::write(main.join("tracked.txt"), "tracked")?;
    git(&main, &["add", "tracked.txt"])?;
    git(
        &main,
        &[
            "-c",
            "user.name=Memory Test",
            "-c",
            "user.email=memory@example.test",
            "commit",
            "-m",
            "initial",
        ],
    )?;
    git(
        &main,
        &[
            "worktree",
            "add",
            "--detach",
            linked.to_str().ok_or("missing fixture value")?,
        ],
    )?;
    git(
        root.path(),
        &[
            "clone",
            main.to_str().ok_or("missing fixture value")?,
            clone.to_str().ok_or("missing fixture value")?,
        ],
    )?;

    let runner = SystemProcessRunner;
    let main_scope = resolve_project(&main, &runner)?;
    let linked_scope = resolve_project(&linked, &runner)?;
    let cloned_scope = resolve_project(&clone, &runner)?;

    assert_eq!(main_scope, linked_scope);
    assert_ne!(main_scope, cloned_scope);
    assert!(main_scope.key().as_str().starts_with("project_"));
    assert_eq!(main_scope.key().as_str().len(), 72);
    Ok(())
}

#[test]
fn invokes_git_with_separate_arguments_at_the_requested_working_directory()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory = tempfile::tempdir()?;
    let common = directory.path().join("common.git");
    fs::create_dir(&common)?;
    let runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);

    resolve_project(directory.path(), &runner)?;

    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls.first().ok_or("missing fixture element")?.program,
        OsStr::new("git")
    );
    assert_eq!(
        calls.first().ok_or("missing fixture element")?.arguments,
        [
            OsString::from("rev-parse"),
            OsString::from("--path-format=absolute"),
            OsString::from("--git-common-dir"),
        ]
    );
    assert_eq!(
        calls
            .first()
            .ok_or("missing fixture element")?
            .current_directory
            .as_deref(),
        Some(directory.path())
    );
    Ok(())
}

#[test]
fn rejects_invalid_project_scope_outputs() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let outside_git = tempfile::tempdir()?;
    assert_scope_unavailable(resolve_project(outside_git.path(), &SystemProcessRunner));

    let existing = tempfile::tempdir()?;
    let nonexistent = existing.path().join("missing.git");
    let cases = [
        ("empty output", FakeResponse::success(Vec::new())),
        ("whitespace output", FakeResponse::success(b" \n".to_vec())),
        ("relative path", FakeResponse::success(b".git\n".to_vec())),
        (
            "non-canonical path",
            FakeResponse::success(format!("{}\n", nonexistent.display())),
        ),
        (
            "ambiguous multiple paths",
            FakeResponse::success(format!(
                "{}\n{}\n",
                existing.path().display(),
                existing.path().display()
            )),
        ),
        ("malformed output", FakeResponse::success(vec![0xff, b'\n'])),
        ("nonzero process", FakeResponse::failure(128, Vec::new())),
    ];

    for (bypass, response) in cases {
        let runner = FakeProcessRunner::with_responses([response]);
        let error = resolve_project(existing.path(), &runner)
            .err()
            .ok_or("expected operation failure")?;
        assert_eq!(error.class(), MemoryErrorClass::Rejection, "{bypass}");
        assert_eq!(error.code(), "scope_unavailable", "{bypass}");
        assert!(!error.to_string().contains("missing.git"), "{bypass}");
    }
    Ok(())
}

#[test]
fn classifies_a_missing_git_program_as_unavailable()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let directory = tempfile::tempdir()?;
    let runner = FakeProcessRunner::with_responses([FakeResponse::missing()]);

    let error = resolve_project(directory.path(), &runner)
        .err()
        .ok_or("expected operation failure")?;

    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(error.code(), "scope_unavailable");
    Ok(())
}

fn assert_scope_unavailable<T: std::fmt::Debug>(result: Result<T, agent_memory::MemoryError>) {
    assert_eq!(
        result.err().as_ref().map(agent_memory::MemoryError::code),
        Some("scope_unavailable")
    );
}

#[test]
fn process_runner_interface_remains_object_safe() {
    fn accepts_runner(_: &dyn ProcessRunner) {}
    accepts_runner(&SystemProcessRunner);
}
