use super::support::*;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

#[test]
fn preparation_rejects_before_any_store_is_opened()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
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
        let error = prepare_admission(bytes, *authorization)
            .err()
            .ok_or("expected operation failure")?;
        assert_eq!(error.code(), *expected);
        assert!(!root.exists());
    }

    fs::write(&root, b"existing store sentinel")?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o640))?;
    let metadata = fs::metadata(&root)?;
    let before = (fs::read(&root)?, metadata.mode(), metadata.ino());
    for (bytes, authorization, expected) in &cases {
        let error = prepare_admission(bytes, *authorization)
            .err()
            .ok_or("expected operation failure")?;
        assert_eq!(error.code(), *expected);
        let metadata = fs::metadata(&root)?;
        assert_eq!((fs::read(&root)?, metadata.mode(), metadata.ino()), before);
    }
    Ok(())
}
