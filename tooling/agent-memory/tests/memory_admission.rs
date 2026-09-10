#![cfg(test)]

#[path = "memory_admission/concurrency.rs"]
mod concurrency;
#[path = "memory_admission/preparation.rs"]
mod preparation;
#[path = "memory_admission/shell_commands.rs"]
mod shell_commands;
#[path = "memory_admission/support.rs"]
mod support;

use support::*;

#[test]
fn admits_a_project_draft_without_an_explicit_scope()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    initialize_repository(fixture.path())?;
    let root = fixture.path().join("store");
    let store = Store::open(&MemoryRoot::new(&root)?)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;

    let result = admit(
        &draft(
            None,
            "invariant",
            "Project memory.",
            "git-file",
            "proof.txt",
        ),
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ExplicitRequest,
        ),
    )?;

    let id = stored_id(result, false)?;
    let entries = store.list()?;
    assert_eq!(entries.entries().len(), 1);
    let entry = &entries.entries().first().ok_or("missing fixture element")?;
    assert_eq!(entry.id().as_str(), id);
    assert_eq!(entry.created_at().as_str(), "2026-08-29T12:00:00Z");
    assert_eq!(
        entry.proof().established_at().as_str(),
        "2026-08-29T12:00:00Z"
    );
    assert_eq!(
        entry
            .proof()
            .sources()
            .first()
            .ok_or("missing fixture element")?
            .fingerprint()
            .as_str(),
        "sha256:c1cda26362828b69266512052b97cb3729e3b052e4ade47c0a1e3383defe73c7"
    );
    assert!(matches!(entry.scope(), EntryScope::Project(_)));
    assert!(root.join("entries/project").read_dir()?.next().is_some());
    Ok(())
}

#[test]
fn admits_user_scope_only_after_an_explicit_authorization()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("store");
    let store = Store::open(&MemoryRoot::new(&root)?)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;
    let bytes = draft(
        Some("user"),
        "invariant",
        "User memory.",
        "user-decision",
        "decision:user-memory",
    );

    let refused = admit(
        &bytes,
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ImplicitProposal,
        ),
    )?;
    assert_rejected(&refused, "admission_not_authorized");
    assert!(store.list()?.entries().is_empty());

    let stored = admit(
        &bytes,
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::AcceptedProposal,
        ),
    )?;
    stored_id(stored, false)?;
    assert_eq!(store.list()?.entries().len(), 1);
    Ok(())
}

#[test]
fn rejects_invalid_input_before_scope_or_source_resolution()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let store = Store::open(&MemoryRoot::new(fixture.path().join("store"))?)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;

    let result = admit(
        b"not: [valid",
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ExplicitRequest,
        ),
    )?;

    assert_rejected(&result, "malformed_yaml");
    assert!(store.list()?.entries().is_empty());
    Ok(())
}

#[test]
fn rejects_the_default_project_scope_outside_git()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let store = Store::open(&MemoryRoot::new(fixture.path().join("store"))?)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;

    let result = admit(
        &draft(
            None,
            "invariant",
            "Project memory.",
            "user-decision",
            "decision:project-memory",
        ),
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ExplicitRequest,
        ),
    )?;

    assert_rejected(&result, "scope_unavailable");
    assert!(store.list()?.entries().is_empty());
    Ok(())
}

#[test]
fn rejects_an_unverified_source_before_commit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let store = Store::open(&MemoryRoot::new(fixture.path().join("store"))?)?;
    let processes = SystemProcessRunner;
    let clock = FixedClock::new()?;
    let missing = fixture.path().join("missing-proof");

    let result = admit(
        &draft(
            Some("user"),
            "invariant",
            "Unverified memory.",
            "local-file",
            missing.to_str().ok_or("missing fixture value")?,
        ),
        context(
            &store,
            fixture.path(),
            &clock,
            &processes,
            AdmissionAuthorization::ExplicitRequest,
        ),
    )?;

    assert_rejected(&result, "source_invalid");
    assert!(store.list()?.entries().is_empty());
    Ok(())
}
