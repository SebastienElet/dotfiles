use super::support::Fixture;
use super::{REQUIRED_SOURCES, arnes, configured_fixture, generate, git};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn records_metadata_sources_and_symlink_identity() {
    let fixture = configured_fixture();
    fixture.write_repository("harness/rules/source.md", "rule\n");
    symlink(
        "source.md",
        fixture.repository().join("harness/rules/adapter.md"),
    )
    .unwrap();

    generate(&fixture);
    let manifest =
        fs::read_to_string(fixture.repository().join(".harness-export/00-MANIFEST.md")).unwrap();

    assert!(manifest.contains("Format version: `1`"));
    assert!(manifest.contains("Repository state at generation (informational): `dirty`"));
    assert!(manifest.contains("Metadata SHA256: `"));
    assert!(manifest.contains("| Source | Kind | Bundle | SHA256 | Bytes | Lines |"));
    for (path, _) in REQUIRED_SOURCES {
        assert!(manifest.contains(&format!("| {path} | file |")), "{path}");
    }
    assert!(manifest.contains("| harness/rules/adapter.md | symlink -> source.md |"));
}

#[test]
fn check_detects_metadata_tampering_without_writing() {
    let fixture = configured_fixture();
    generate(&fixture);
    let manifest_path = fixture.repository().join(".harness-export/00-MANIFEST.md");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    let tampered = manifest
        .lines()
        .map(|line| {
            if line.starts_with("- Git commit at generation (informational):") {
                "- Git commit at generation (informational): `deadbeef`"
            } else {
                line
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    fs::write(&manifest_path, &tampered).unwrap();

    let output = arnes(&fixture, &["export", "--check"]);

    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("metadata integrity"));
    assert_eq!(fs::read_to_string(manifest_path).unwrap(), tampered);
}

#[test]
fn generation_without_a_commit_records_unavailable() {
    let fixture = Fixture::new();
    for (path, contents) in REQUIRED_SOURCES {
        fixture.write_repository(path, contents);
    }
    git(&fixture, &["init", "-q"]);

    generate(&fixture);

    let manifest =
        fs::read_to_string(fixture.repository().join(".harness-export/00-MANIFEST.md")).unwrap();
    assert!(manifest.contains("Git commit at generation (informational): `unavailable`"));
}
