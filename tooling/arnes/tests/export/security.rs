use super::{arnes, configured_fixture};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn obvious_sensitive_paths_fail_closed() {
    for path in [
        "harness/skills/alpha/.env",
        "harness/skills/alpha/credentials.yaml",
        "harness/skills/alpha/prod-credentials.yml",
        "harness/skills/alpha/secret.yaml",
        "harness/skills/alpha/api_key.yaml",
    ] {
        let fixture = configured_fixture();
        fixture.write_repository(path, "api_token: super-secret-value\n");

        let output = arnes(&fixture, &["export"]);

        assert!(!output.status.success(), "{path}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("sensitive"));
        assert!(!fixture.repository().join(".harness-export").exists());
    }
}

#[test]
fn selected_links_cannot_reintroduce_ignored_sources() {
    let fixture = configured_fixture();
    fixture.write_repository(
        ".gitignore",
        ".harness-export/\nharness/skills/alpha/.claude-flow/\n",
    );
    fixture.write_repository(
        "harness/skills/alpha/.claude-flow/credentials.yaml",
        "api_key: super-secret-value\n",
    );
    symlink(
        ".claude-flow/credentials.yaml",
        fixture.repository().join("harness/skills/alpha/public.md"),
    )
    .unwrap();

    let output = arnes(&fixture, &["export"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unselected target"));
}

#[test]
fn ignored_relevant_sources_fail_closed() {
    let fixture = configured_fixture();
    fixture.write_repository(".gitignore", ".harness-export/\nharness/skills/ignored/\n");
    fixture.write_repository("harness/skills/ignored/SKILL.md", "ignored\n");

    let output = arnes(&fixture, &["export"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ignored harness source"));
}

#[test]
fn external_symlinks_and_hardlinks_fail_closed() {
    for hardlink in [false, true] {
        let fixture = configured_fixture();
        let selected = fixture.repository().join("harness/skills/alpha/SKILL.md");
        let outside = fixture.home().join("sensitive");
        fs::write(&outside, "sensitive\n").unwrap();
        fs::remove_file(&selected).unwrap();
        if hardlink {
            fs::hard_link(&outside, &selected).unwrap();
        } else {
            symlink(&outside, &selected).unwrap();
        }

        let output = arnes(&fixture, &["export"]);

        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("refusing"));
    }
}

#[test]
fn export_path_symlinks_and_regular_files_are_refused_without_mutation() {
    for symlink_path in [false, true] {
        let fixture = configured_fixture();
        let export = fixture.repository().join(".harness-export");
        let sentinel = fixture.home().join("sentinel");
        fs::create_dir(&sentinel).unwrap();
        fs::write(sentinel.join("value"), "unchanged\n").unwrap();
        if symlink_path {
            symlink(&sentinel, &export).unwrap();
        } else {
            fs::write(&export, "unchanged\n").unwrap();
        }

        let output = arnes(&fixture, &["export"]);

        assert!(!output.status.success());
        assert_eq!(fs::read(sentinel.join("value")).unwrap(), b"unchanged\n");
        if !symlink_path {
            assert_eq!(fs::read(export).unwrap(), b"unchanged\n");
        }
    }
}
