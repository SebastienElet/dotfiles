use super::support::*;
use std::fs;

#[test]
fn rejected_input_never_creates_or_opens_the_store() {
    let fixture = CliFixture::new();
    let secret = "ghp_admission_order_redaction_sentinel";
    let sensitive = fixture.git_draft("invariant", secret, "sensitive admission");

    let malformed = fixture.run(["admit", "--format", "json"], b"not: [valid");
    assert_error(&malformed, 2, "malformed_yaml");
    assert!(!fixture.root().exists());

    let rejected = fixture.run(["admit", "--format", "json"], &sensitive);
    assert_error(&rejected, 2, "sensitive_content");
    assert!(!fixture.root().exists());
}

#[test]
fn unavailable_root_cannot_mask_a_parse_rejection() {
    let fixture = CliFixture::new();
    fs::write(fixture.root(), b"unavailable root sentinel").unwrap();
    let before = tree_snapshot(fixture.root());

    let output = fixture.run(["admit", "--format", "json"], b"not: [valid");

    assert_error(&output, 2, "malformed_yaml");
    assert_eq!(tree_snapshot(fixture.root()), before);
}

#[test]
fn rejected_input_preserves_existing_store_bytes_modes_and_inodes() {
    let fixture = CliFixture::new();
    let valid = fixture.git_draft("invariant", "Stored memory.", "stored memory");
    assert_exit(&fixture.run(["admit", "--format", "json"], &valid), 0);
    let secret = "ghp_existing_store_redaction_sentinel";
    let sensitive = fixture.git_draft("invariant", secret, "sensitive admission");
    let cases = [
        (b"not: [valid".as_slice(), "malformed_yaml"),
        (sensitive.as_slice(), "sensitive_content"),
    ];

    for (bytes, expected) in cases {
        make_store_modes_non_private(fixture.root());
        let before = tree_snapshot(fixture.root());
        let output = fixture.run(["admit", "--format", "json"], bytes);
        assert_error(&output, 2, expected);
        assert_eq!(tree_snapshot(fixture.root()), before, "{expected}");
    }
}
