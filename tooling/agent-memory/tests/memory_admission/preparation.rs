use super::support::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[test]
fn preparation_rejects_before_any_store_is_opened() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("store");
    let sensitive = draft(
        Some("user"),
        "invariant",
        "token=credential",
        "user-decision",
        "decision:sensitive",
    );
    let unauthorized = draft(
        Some("user"),
        "invariant",
        "Unauthorized memory.",
        "user-decision",
        "decision:unauthorized",
    );
    let cases = [
        (
            b"not: [valid".as_slice(),
            AdmissionAuthorization::ExplicitRequest,
            "malformed_yaml",
        ),
        (
            sensitive.as_slice(),
            AdmissionAuthorization::ExplicitRequest,
            "sensitive_content",
        ),
        (
            unauthorized.as_slice(),
            AdmissionAuthorization::ImplicitProposal,
            "admission_not_authorized",
        ),
    ];

    for (bytes, authorization, expected) in &cases {
        let error = prepare_admission(bytes, *authorization).unwrap_err();
        assert_eq!(error.code(), *expected);
        assert!(!root.exists());
    }

    fs::write(&root, b"existing store sentinel").unwrap();
    fs::set_permissions(&root, fs::Permissions::from_mode(0o640)).unwrap();
    let metadata = fs::metadata(&root).unwrap();
    let before = (fs::read(&root).unwrap(), metadata.mode(), metadata.ino());
    for (bytes, authorization, expected) in &cases {
        let error = prepare_admission(bytes, *authorization).unwrap_err();
        assert_eq!(error.code(), *expected);
        let metadata = fs::metadata(&root).unwrap();
        assert_eq!(
            (fs::read(&root).unwrap(), metadata.mode(), metadata.ino()),
            before
        );
    }
}
