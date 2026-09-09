use super::*;
use std::{fs, os::unix::fs::symlink};

#[test]
fn normalizes_actual_cat_targets_and_redacts_arbitrary_arguments() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir_all(root.join("src/auth")).unwrap();
    fs::create_dir_all(root.join("other/src/auth")).unwrap();
    fs::write(root.join("src/auth/session.ts"), "real").unwrap();
    fs::write(root.join("other/src/auth/session.ts"), "shadow").unwrap();
    symlink(root.join("src/auth/session.ts"), root.join("target")).unwrap();
    let args = ["src/auth/session.ts".into()];
    assert_eq!(
        public_arguments("cat", &args, root, &root.join("other")),
        ["<other>"]
    );
    assert_eq!(
        public_arguments("cat", &["session.ts".into()], root, &root.join("src/auth")),
        ["src/auth/session.ts"]
    );
    assert_eq!(
        public_arguments("cat", &["target".into()], root, root),
        ["src/auth/session.ts"]
    );
    assert_eq!(
        public_arguments("cat", &["missing".into()], root, root),
        ["<other>"]
    );
    assert_eq!(
        public_arguments("colgrep-search", &["secret".into()], root, root),
        ["<other>"]
    );
}

#[test]
fn synthetic_tools_preserve_outputs_and_failure_statuses() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join("first"), "é").unwrap();
    fs::write(root.path().join("second"), "\n").unwrap();
    assert_eq!(
        invoke("cat", &["first".into(), "second".into()], root.path()),
        ("é\n".into(), 0)
    );
    assert_eq!(
        invoke("cat", &["first".into(), "missing".into()], root.path()),
        (String::new(), 1)
    );
    assert!(
        invoke("rg", &["FEATURE_FLAG_DISABLED".into()], root.path())
            .0
            .contains("src/flags.ts")
    );
    assert_eq!(invoke("fd", &[], root.path()).1, 0);
    assert_eq!(invoke("rg", &["--files".into()], root.path()).1, 0);
    assert_eq!(
        invoke("colgrep-search", &["question".into()], root.path()).1,
        0
    );
    assert_eq!(
        invoke("colgrep-search", &["--help".into()], root.path()).1,
        64
    );
}
