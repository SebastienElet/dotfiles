use super::support::*;
use rustix::fs::FlockOperation;
use std::fs::File;
#[test]
fn a_store_lock_timeout_is_a_conflict_at_the_cli_boundary()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = CliFixture::new()?;
    let first = CliFixture::git_draft("invariant", "Initial lock memory.", "initial lock memory")?;
    assert_exit(&fixture.run(["admit", "--format", "json"], &first)?, 0);
    let second = CliFixture::git_draft("invariant", "Blocked lock memory.", "blocked lock memory")?;
    let root = File::open(fixture.root())?;
    rustix::fs::flock(&root, FlockOperation::LockExclusive)?;
    let output = fixture.run(["admit", "--format", "json"], &second)?;
    assert_error(&output, 3, "store_lock_timeout")?;
    Ok(())
}
