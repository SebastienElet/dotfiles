use agent_memory::{
    AdmissionAuthorization, SourceContext, SystemProcessRunner, parse_draft, resolve_sources,
    validate_draft,
};
use std::fs;
use std::path::Path;
use std::process::Command;

fn draft(scope: &str, locator: &str) -> agent_memory::ValidatedDraft {
    let yaml = format!(
        "schema_version: 1\nkind: invariant\nstatement: A tracked proof establishes this memory.\nscope: {scope}\nretrieval_terms:\n  - tracked proof\nproof:\n  summary: The tracked file is authoritative.\n  sources:\n    - kind: git-file\n      locator: {}\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the tracked file remain authoritative?\n    valid_when: The tracked file retains the requirement.\n  outcomes:\n    valid: The proof is unchanged.\n    invalidated: The proof changed.\n",
        serde_json::to_string(locator).unwrap()
    );
    validate_draft(
        parse_draft(yaml.as_bytes()).unwrap(),
        AdmissionAuthorization::AcceptedProposal,
    )
    .unwrap()
}

fn initialize_repository(path: &Path) {
    fs::create_dir_all(path).unwrap();
    git(path, &["init"]);
    fs::create_dir(path.join("docs")).unwrap();
    fs::write(path.join("docs/proof.txt"), "same proof").unwrap();
    git(path, &["add", "docs/proof.txt"]);
}

fn commit_repository(path: &Path) {
    git(
        path,
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
}

fn git(directory: &Path, arguments: &[&str]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .status()
            .unwrap()
            .success()
    );
}

#[test]
fn refuses_untracked_literal_names_that_git_would_treat_as_pathspecs() {
    let temporary = tempfile::tempdir().unwrap();
    initialize_repository(temporary.path());
    fs::write(temporary.path().join("*.txt"), "untracked wildcard").unwrap();
    fs::write(
        temporary.path().join(":(glob)*.txt"),
        "untracked pathspec magic",
    )
    .unwrap();
    let runner = SystemProcessRunner;
    let context = SourceContext::new(temporary.path(), &runner, &runner);

    for locator in ["*.txt", ":(glob)*.txt"] {
        let error = resolve_sources(draft("project", locator), &context).unwrap_err();
        assert_eq!(error.code(), "source_invalid", "{locator}");
    }
}

#[test]
fn persists_a_project_bound_repository_relative_locator_from_nested_cwd() {
    let temporary = tempfile::tempdir().unwrap();
    initialize_repository(temporary.path());
    let runner = SystemProcessRunner;
    let docs = temporary.path().join("docs");
    let context = SourceContext::new(&docs, &runner, &runner);

    let resolved = resolve_sources(draft("project", "proof.txt"), &context).unwrap();

    assert_eq!(resolved.sources()[0].locator(), "docs/proof.txt");
}

#[test]
fn linked_worktrees_share_the_same_canonical_git_locator() {
    let temporary = tempfile::tempdir().unwrap();
    let main = temporary.path().join("main");
    let linked = temporary.path().join("linked");
    initialize_repository(&main);
    commit_repository(&main);
    git(
        &main,
        &["worktree", "add", "--detach", linked.to_str().unwrap()],
    );
    let runner = SystemProcessRunner;
    let main_docs = main.join("docs");
    let linked_docs = linked.join("docs");
    let main_context = SourceContext::new(&main_docs, &runner, &runner);
    let linked_context = SourceContext::new(&linked_docs, &runner, &runner);

    let from_main = resolve_sources(draft("project", "proof.txt"), &main_context).unwrap();
    let from_linked = resolve_sources(draft("project", "proof.txt"), &linked_context).unwrap();

    assert_eq!(from_main.sources()[0], from_linked.sources()[0]);
    assert_eq!(from_main.sources()[0].locator(), "docs/proof.txt");
}

#[test]
fn user_scope_git_sources_do_not_rebind_across_projects() {
    let temporary = tempfile::tempdir().unwrap();
    let first = temporary.path().join("first");
    let second = temporary.path().join("second");
    initialize_repository(&first);
    initialize_repository(&second);
    let runner = SystemProcessRunner;
    let first_docs = first.join("docs");
    let first_context = SourceContext::new(&first_docs, &runner, &runner);
    let second_context = SourceContext::new(&second, &runner, &runner);
    let resolved = resolve_sources(draft("user", "proof.txt"), &first_context).unwrap();

    resolved.recheck_sources(&second_context).unwrap();

    assert_eq!(resolved.sources()[0].locator(), "docs/proof.txt");
}
