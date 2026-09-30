use super::{prepare, publish_link};
use crate::Roots;
use crate::manifest::{self, Agent, Scope};
use crate::sync::SyncResource;
use crate::sync::selection;
use std::fs;
use std::os::unix::fs::symlink;

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn prepared_rule(
    root: &std::path::Path,
) -> Result<super::PreparedLink, Box<dyn std::error::Error>> {
    let repository = root.join("repository");
    let home = root.join("home");
    fs::create_dir_all(repository.join("harness/rules"))?;
    fs::create_dir_all(&home)?;
    fs::write(repository.join("harness/rules/rule.md"), "rule")?;
    let manifest = manifest::parse(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources:\n  - id: rule\n    kind: rules\n    agent: claude\n    scope: user\n    source: { root: repository, path: harness/rules/rule.md }\n    destination: { root: home, path: .claude/rules/rule.md }\n",
    )?;
    let roots = Roots::new(repository, home);
    let mut selected = selection::select(
        &roots,
        &manifest,
        SyncResource::Rules,
        Agent::Claude,
        Scope::User,
    )
    .map_err(|entry| entry.message)?;
    let intent = selected.pop().ok_or("selection")?;
    prepare(intent).map_err(|entry| entry.message.into())
}

#[test]
fn publication_never_replaces_a_destination_created_after_validation() -> TestResult {
    let root = tempfile::tempdir()?;
    let link = prepared_rule(root.path())?;
    let destination = link.intent.root.join(&link.intent.destination);
    fs::create_dir_all(destination.parent().ok_or("parent")?)?;
    fs::write(&destination, "newer local file")?;
    assert!(publish_link(&link).is_err());
    assert_eq!(fs::read(destination)?, b"newer local file");
    Ok(())
}

#[test]
fn publication_refuses_a_parent_replaced_by_a_symlink_after_validation() -> TestResult {
    let root = tempfile::tempdir()?;
    let link = prepared_rule(root.path())?;
    let outside = tempfile::tempdir()?;
    symlink(outside.path(), link.intent.root.join(".claude"))?;
    assert!(publish_link(&link).is_err());
    assert_eq!(fs::read_dir(outside.path())?.count(), 0);
    Ok(())
}

#[test]
fn publication_refuses_source_replacement_after_validation() -> TestResult {
    let root = tempfile::tempdir()?;
    let link = prepared_rule(root.path())?;
    let replacement = root.path().join("replacement");
    fs::write(&replacement, "replacement")?;
    fs::rename(replacement, &link.source)?;
    assert!(publish_link(&link).is_err());
    assert!(!link.intent.root.join(".claude").exists());
    assert_eq!(fs::read(&link.source)?, b"replacement");
    Ok(())
}

#[test]
fn current_link_changed_after_validation_is_not_reported_conforming() -> TestResult {
    let root = tempfile::tempdir()?;
    let link = prepared_rule(root.path())?;
    let destination = link.intent.root.join(&link.intent.destination);
    fs::create_dir_all(destination.parent().ok_or("parent")?)?;
    symlink(&link.source, &destination)?;
    let current = prepare(link.intent).map_err(|entry| entry.message)?;
    fs::remove_file(&destination)?;
    fs::write(&destination, "local")?;
    assert!(publish_link(&current).is_err());
    assert_eq!(fs::read(destination)?, b"local");
    Ok(())
}
