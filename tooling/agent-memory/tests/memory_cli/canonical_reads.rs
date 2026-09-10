use super::support::*;
use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
#[test]
fn audit_and_retrieval_leave_an_absent_store_absent()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let audit_fixture = CliFixture::new()?;
    let audit = audit_fixture.run(["audit", "--format", "json"], b"")?;
    assert_exit(&audit, 0);
    assert_eq!(
        (*(stdout_json(&audit)?)
            .get("entries")
            .ok_or(concat!("missing fixture index ", stringify!("entries")))?),
        Value::Array(Vec::new())
    );
    assert!(!audit_fixture.root().exists());
    let retrieval_fixture = CliFixture::new()?;
    let retrieval = retrieval_fixture.run(
        ["retrieve", "--query-stdin", "--format", "json"],
        b"no stored memory",
    )?;
    assert_exit(&retrieval, 0);
    assert_eq!(
        (*(stdout_json(&retrieval)?)
            .get("injected")
            .ok_or(concat!("missing fixture index ", stringify!("injected")))?),
        Value::Array(Vec::new())
    );
    assert!(!retrieval_fixture.root().exists());
    Ok(())
}
#[test]
fn audit_refuses_non_private_yaml_without_repairing_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = stored_fixture()?;
    let yaml = only_yaml(fixture.root())?;
    fs::set_permissions(&yaml, fs::Permissions::from_mode(0o644))?;
    let before = tree_snapshot(fixture.root())?;
    let output = fixture.run(["audit", "--include-terminal", "--format", "json"], b"")?;
    assert_error(&output, 4, "store_permissions_unavailable")?;
    assert_eq!(tree_snapshot(fixture.root())?, before);
    Ok(())
}
#[test]
fn retrieval_refuses_non_private_yaml_without_repairing_it()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = stored_fixture()?;
    let yaml = only_yaml(fixture.root())?;
    fs::set_permissions(&yaml, fs::Permissions::from_mode(0o644))?;
    let before = tree_snapshot(fixture.root())?;
    let output = fixture.run(
        ["retrieve", "--query-stdin", "--format", "json"],
        b"stored read-only memory",
    )?;
    assert_error(&output, 4, "store_permissions_unavailable")?;
    assert_eq!(tree_snapshot(fixture.root())?, before);
    Ok(())
}
fn stored_fixture() -> Result<CliFixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let draft = CliFixture::git_draft(
        "invariant",
        "Stored read-only memory.",
        "stored read-only memory",
    )?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &draft)?, 0);
    Ok(fixture)
}
