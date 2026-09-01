#[path = "support/memory.rs"]
mod memory_support;

use agent_memory::{MemoryErrorClass, ProcessRunner, SystemProcessRunner, resolve_project};
use memory_support::{FakeProcessRunner, FakeResponse, git};
use std::ffi::{OsStr, OsString};
use std::fs;

#[test]
fn shares_project_identity_between_linked_worktrees_but_not_clones() {
    let root = tempfile::tempdir().unwrap();
    let main = root.path().join("main");
    let linked = root.path().join("linked");
    let clone = root.path().join("clone");
    fs::create_dir(&main).unwrap();
    git(&main, &["init"]);
    fs::write(main.join("tracked.txt"), "tracked").unwrap();
    git(&main, &["add", "tracked.txt"]);
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
    );
    git(
        &main,
        &["worktree", "add", "--detach", linked.to_str().unwrap()],
    );
    git(
        root.path(),
        &["clone", main.to_str().unwrap(), clone.to_str().unwrap()],
    );

    let runner = SystemProcessRunner;
    let main_scope = resolve_project(&main, &runner).unwrap();
    let linked_scope = resolve_project(&linked, &runner).unwrap();
    let cloned_scope = resolve_project(&clone, &runner).unwrap();

    assert_eq!(main_scope, linked_scope);
    assert_ne!(main_scope, cloned_scope);
    assert!(main_scope.key().as_str().starts_with("project_"));
    assert_eq!(main_scope.key().as_str().len(), 72);
}

#[test]
fn invokes_git_with_separate_arguments_at_the_requested_working_directory() {
    let directory = tempfile::tempdir().unwrap();
    let common = directory.path().join("common.git");
    fs::create_dir(&common).unwrap();
    let runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);

    resolve_project(directory.path(), &runner).unwrap();

    let calls = runner.calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].program, OsStr::new("git"));
    assert_eq!(
        calls[0].arguments,
        [
            OsString::from("rev-parse"),
            OsString::from("--path-format=absolute"),
            OsString::from("--git-common-dir"),
        ]
    );
    assert_eq!(
        calls[0].current_directory.as_deref(),
        Some(directory.path())
    );
}

#[test]
fn rejects_invalid_project_scope_outputs() {
    let outside_git = tempfile::tempdir().unwrap();
    assert_scope_unavailable(resolve_project(outside_git.path(), &SystemProcessRunner));

    let existing = tempfile::tempdir().unwrap();
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
        let error = resolve_project(existing.path(), &runner).unwrap_err();
        assert_eq!(error.class(), MemoryErrorClass::Rejection, "{bypass}");
        assert_eq!(error.code(), "scope_unavailable", "{bypass}");
        assert!(!error.to_string().contains("missing.git"), "{bypass}");
    }
}

#[test]
fn classifies_a_missing_git_program_as_unavailable() {
    let directory = tempfile::tempdir().unwrap();
    let runner = FakeProcessRunner::with_responses([FakeResponse::missing()]);

    let error = resolve_project(directory.path(), &runner).unwrap_err();

    assert_eq!(error.class(), MemoryErrorClass::Unavailable);
    assert_eq!(error.code(), "scope_unavailable");
}

fn assert_scope_unavailable<T: std::fmt::Debug>(result: Result<T, agent_memory::MemoryError>) {
    assert_eq!(result.unwrap_err().code(), "scope_unavailable");
}

#[test]
fn process_runner_interface_remains_object_safe() {
    fn accepts_runner(_: &dyn ProcessRunner) {}
    accepts_runner(&SystemProcessRunner);
}
