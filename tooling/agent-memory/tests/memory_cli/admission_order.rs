use super::support::*;
use std::fs;
#[test]
fn rejected_input_never_creates_or_opens_the_store()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let secret = "ghp_admission_order_redaction_sentinel";
    let sensitive = CliFixture::git_draft("invariant", secret, "sensitive admission")?;
    let malformed = fixture.run(["admit", "--format", "json"], b"not: [valid")?;
    assert_error(&malformed, 2, "malformed_yaml")?;
    assert!(!fixture.root().exists());
    let rejected = fixture.run(["admit", "--format", "json"], &sensitive)?;
    assert_error(&rejected, 2, "sensitive_content")?;
    assert!(!fixture.root().exists());
    Ok(())
}
#[test]
fn unavailable_root_cannot_mask_a_parse_rejection()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    fs::write(fixture.root(), b"unavailable root sentinel")?;
    let before = tree_snapshot(fixture.root())?;
    let output = fixture.run(["admit", "--format", "json"], b"not: [valid")?;
    assert_error(&output, 2, "malformed_yaml")?;
    assert_eq!(tree_snapshot(fixture.root())?, before);
    Ok(())
}
#[test]
fn rejected_input_preserves_existing_store_bytes_modes_and_inodes()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let valid = CliFixture::git_draft("invariant", "Stored memory.", "stored memory")?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &valid)?, 0);
    let secret = "ghp_existing_store_redaction_sentinel";
    let sensitive = CliFixture::git_draft("invariant", secret, "sensitive admission")?;
    let cases = [
        (b"not: [valid".as_slice(), "malformed_yaml"),
        (sensitive.as_slice(), "sensitive_content"),
    ];
    let _: () = for (bytes, expected) in cases {
        make_store_modes_non_private(fixture.root())?;
        let before = tree_snapshot(fixture.root())?;
        let output = fixture.run(["admit", "--format", "json"], bytes)?;
        assert_error(&output, 2, expected)?;
        assert_eq!(tree_snapshot(fixture.root())?, before, "{expected}");
    };
    Ok(())
}
#[test]
fn scope_and_source_rejections_precede_store_creation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let outside_git = tempfile::tempdir()?;
    let project = CliFixture::git_draft("invariant", "Missing project scope.", "missing project")?;
    let missing_source = String::from_utf8(CliFixture::git_draft(
        "invariant",
        "Missing source proof.",
        "missing source",
    )?)?
    .replace("locator: proof.txt", "locator: absent.txt")
    .into_bytes();
    let scope = CliFixture::run_from_with_root(
        outside_git.path(),
        ["admit", "--format", "json"],
        &project,
        fixture.root(),
    )?;
    assert_error(&scope, 2, "scope_unavailable")?;
    assert!(!fixture.root().exists());
    let source = fixture.run(["admit", "--format", "json"], &missing_source)?;
    assert_error(&source, 2, "source_invalid")?;
    assert!(!fixture.root().exists());
    let policy = fixture.run(
        ["admit", "--format", "json"],
        &CliFixture::user_git_draft("Unsupported user Git proof.")?,
    )?;
    assert_error(&policy, 2, "source_invalid")?;
    assert!(!fixture.root().exists());
    Ok(())
}
#[test]
fn an_unresolvable_current_directory_precedes_store_creation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let directory = tempfile::tempdir()?;
    let cwd = directory.path().to_owned();
    let project = CliFixture::git_draft("invariant", "Missing current directory.", "missing cwd")?;
    let output = fixture.run_after_cwd_removal(&cwd, ["admit", "--format", "json"], &project)?;
    assert_error(&output, 2, "scope_unavailable")?;
    assert!(!fixture.root().exists());
    Ok(())
}
#[test]
fn unavailable_store_cannot_mask_scope_source_or_policy_rejections()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let outside_git = tempfile::tempdir()?;
    fs::write(fixture.root(), b"unavailable root sentinel")?;
    let before = tree_snapshot(fixture.root())?;
    let project = CliFixture::git_draft("invariant", "Missing project scope.", "missing project")?;
    let missing_source = String::from_utf8(CliFixture::git_draft(
        "invariant",
        "Missing source proof.",
        "missing source",
    )?)?
    .replace("locator: proof.txt", "locator: absent.txt")
    .into_bytes();
    let user_git = CliFixture::user_git_draft("Unsupported user Git proof.")?;
    let cases = [
        (outside_git.path(), project.as_slice(), "scope_unavailable"),
        (
            fixture.repository(),
            missing_source.as_slice(),
            "source_invalid",
        ),
        (fixture.repository(), user_git.as_slice(), "source_invalid"),
    ];
    let _: () = for (cwd, draft, expected) in cases {
        let output = CliFixture::run_from_with_root(
            cwd,
            ["admit", "--format", "json"],
            draft,
            fixture.root(),
        )?;
        assert_error(&output, 2, expected)?;
        assert_eq!(tree_snapshot(fixture.root())?, before, "{expected}");
    };
    Ok(())
}
#[test]
fn scope_source_and_policy_rejections_do_not_repair_an_existing_store()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let valid = CliFixture::git_draft("invariant", "Stored memory.", "stored memory")?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &valid)?, 0);
    make_store_modes_non_private(fixture.root())?;
    let before = tree_snapshot(fixture.root())?;
    let outside_git = tempfile::tempdir()?;
    let project = CliFixture::git_draft("invariant", "Missing project scope.", "missing project")?;
    let missing_source = String::from_utf8(CliFixture::git_draft(
        "invariant",
        "Missing source proof.",
        "missing source",
    )?)?
    .replace("locator: proof.txt", "locator: absent.txt")
    .into_bytes();
    let user_git = CliFixture::user_git_draft("Unsupported user Git proof.")?;
    let cases = [
        (outside_git.path(), project.as_slice(), "scope_unavailable"),
        (
            fixture.repository(),
            missing_source.as_slice(),
            "source_invalid",
        ),
        (fixture.repository(), user_git.as_slice(), "source_invalid"),
    ];
    let _: () = for (cwd, draft, expected) in cases {
        let output = CliFixture::run_from_with_root(
            cwd,
            ["admit", "--format", "json"],
            draft,
            fixture.root(),
        )?;
        assert_error(&output, 2, expected)?;
        assert_eq!(tree_snapshot(fixture.root())?, before, "{expected}");
    };
    Ok(())
}
