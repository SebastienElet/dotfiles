#![cfg(test)]

use agent_memory::{
    AdmissionAuthorization, SourceContext, SystemProcessRunner, parse_draft, resolve_sources,
    validate_draft,
};
use std::fs;
use std::path::Path;
use std::process::Command;

fn draft_yaml(scope: &str, locator: &str) -> Vec<u8> {
    format!(
        "schema_version: 1\nkind: invariant\nstatement: A tracked proof establishes this memory.\nscope: {scope}\nretrieval_terms:\n  - tracked proof\nproof:\n  summary: The tracked file is authoritative.\n  sources:\n    - kind: git-file\n      locator: {}\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the tracked file remain authoritative?\n    valid_when: The tracked file retains the requirement.\n  outcomes:\n    valid: The proof is unchanged.\n    invalidated: The proof changed.\n",
        serde_json::Value::from(locator)
    )
    .into_bytes()
}

fn draft(
    scope: &str,
    locator: &str,
) -> Result<agent_memory::ValidatedDraft, agent_memory::MemoryError> {
    validate_draft(
        parse_draft(&draft_yaml(scope, locator))?,
        AdmissionAuthorization::AcceptedProposal,
    )
}

fn initialize_repository(path: &Path) -> std::io::Result<()> {
    fs::create_dir_all(path)?;
    git(path, &["init"])?;
    fs::create_dir(path.join("docs"))?;
    fs::write(path.join("docs/proof.txt"), "same proof")?;
    git(path, &["add", "docs/proof.txt"])?;
    Ok(())
}

fn commit_repository(path: &Path) -> std::io::Result<()> {
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
    )?;
    Ok(())
}

fn git(directory: &Path, arguments: &[&str]) -> std::io::Result<()> {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .status()?
            .success()
    );
    Ok(())
}

#[test]
fn refuses_untracked_literal_names_that_git_would_treat_as_pathspecs()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let temporary = tempfile::tempdir()?;
    initialize_repository(temporary.path())?;
    fs::write(temporary.path().join("*.txt"), "untracked wildcard")?;
    fs::write(
        temporary.path().join(":(glob)*.txt"),
        "untracked pathspec magic",
    )?;
    let runner = SystemProcessRunner;
    let context = SourceContext::new(temporary.path(), &runner, &runner);

    for locator in ["*.txt", ":(glob)*.txt"] {
        let error = resolve_sources(draft("project", locator)?, &context)
            .err()
            .ok_or("expected operation failure")?;
        assert_eq!(error.code(), "source_invalid", "{locator}");
    }
    Ok(())
}

#[test]
fn persists_a_project_bound_repository_relative_locator_from_nested_cwd()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let temporary = tempfile::tempdir()?;
    initialize_repository(temporary.path())?;
    let runner = SystemProcessRunner;
    let docs = temporary.path().join("docs");
    let context = SourceContext::new(&docs, &runner, &runner);

    let resolved = resolve_sources(draft("project", "proof.txt")?, &context)?;

    assert_eq!(
        resolved
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .locator(),
        "docs/proof.txt"
    );
    Ok(())
}

#[test]
fn linked_worktrees_share_the_same_canonical_git_locator()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let temporary = tempfile::tempdir()?;
    let main = temporary.path().join("main");
    let linked = temporary.path().join("linked");
    initialize_repository(&main)?;
    commit_repository(&main)?;
    git(
        &main,
        &[
            "worktree",
            "add",
            "--detach",
            linked.to_str().ok_or("missing fixture value")?,
        ],
    )?;
    let runner = SystemProcessRunner;
    let main_docs = main.join("docs");
    let linked_docs = linked.join("docs");
    let main_context = SourceContext::new(&main_docs, &runner, &runner);
    let linked_context = SourceContext::new(&linked_docs, &runner, &runner);

    let from_main = resolve_sources(draft("project", "proof.txt")?, &main_context)?;
    let from_linked = resolve_sources(draft("project", "proof.txt")?, &linked_context)?;

    assert_eq!(
        from_main
            .sources()
            .first()
            .ok_or("missing fixture element")?,
        from_linked
            .sources()
            .first()
            .ok_or("missing fixture element")?
    );
    assert_eq!(
        from_main
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .locator(),
        "docs/proof.txt"
    );
    Ok(())
}

#[test]
fn user_scope_git_sources_are_rejected_by_admission_validation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let parsed = parse_draft(&draft_yaml("user", "proof.txt"))?;
    let error = validate_draft(parsed, AdmissionAuthorization::AcceptedProposal)
        .err()
        .ok_or("expected operation failure")?;

    assert_eq!(error.code(), "source_invalid");
    assert_eq!(error.field(), "proof.sources");
    Ok(())
}
