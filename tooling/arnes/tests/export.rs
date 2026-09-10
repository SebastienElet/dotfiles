#![cfg(test)]
#[path = "export/manifest.rs"]
mod manifest;
#[path = "export/security.rs"]
mod security;
pub mod support;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use support::Fixture;
const REQUIRED_SOURCES: [(&str, &str); 4] = [
    ("harness/AGENTS.md", "global instructions\n"),
    ("harness/SOUL.md", "identity\n"),
    ("harness/USER.md", "preferences\n"),
    ("home/.arnes.yaml", "version: 1\nhooks: []\n"),
];
pub(crate) fn configured_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    for (path, contents) in REQUIRED_SOURCES {
        fixture.write_repository(path, contents)?;
    }
    fixture.write_repository(".gitignore", ".harness-export/\n")?;
    fixture.write_repository("harness/skills/alpha/SKILL.md", "alpha\n")?;
    fixture.write_repository("harness/skills/beta/SKILL.md", "beta\n")?;
    fixture.write_repository("harness/assets/example.txt", "asset\n")?;
    git(&fixture, &["init", "-q"])?;
    git(&fixture, &["add", "."])?;
    git(
        &fixture,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.com",
            "commit",
            "-qm",
            "fixture",
        ],
    )?;
    Ok(fixture)
}
pub(crate) fn git(
    fixture: &Fixture,
    arguments: &[&str],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(fixture.repository())
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
pub(crate) fn arnes(
    fixture: &Fixture,
    arguments: &[&str],
) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
    Ok(Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(arguments)
        .current_dir(fixture.repository())
        .env("HOME", fixture.home())
        .output()?)
}
pub(crate) fn generate(fixture: &Fixture) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = arnes(fixture, &["export"])?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
fn export_files(
    fixture: &Fixture,
) -> Result<BTreeMap<PathBuf, Vec<u8>>, Box<dyn std::error::Error + Send + Sync>> {
    fs::read_dir(fixture.repository().join(".harness-export"))?
        .map(
            |entry| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok({
                    let path = entry?.path();
                    (
                        path.file_name()
                            .ok_or("required test value is missing")?
                            .into(),
                        fs::read(path)?,
                    )
                })
            },
        )
        .collect::<Result<BTreeMap<_, _>, _>>()
}
fn bundle_contents(fixture: &Fixture) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    export_files(fixture)?
        .into_iter()
        .filter(|(path, _)| path != Path::new("00-MANIFEST.md"))
        .map(
            |(_, contents)| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(String::from_utf8(contents)?)
            },
        )
        .collect::<Result<String, _>>()
}
#[test]
fn output_order_is_deterministic_and_duplicate_basenames_keep_their_paths()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    generate(&fixture)?;
    let first = export_files(&fixture)?;
    generate(&fixture)?;
    let second = export_files(&fixture)?;
    let bundles = bundle_contents(&fixture)?;
    let without_manifest = |files: BTreeMap<PathBuf, Vec<u8>>| {
        files
            .into_iter()
            .filter(|(path, _)| path != Path::new("00-MANIFEST.md"))
            .collect::<BTreeMap<_, _>>()
    };
    assert_eq!(without_manifest(first), without_manifest(second));
    let _: () = for path in [
        "harness/skills/alpha/SKILL.md",
        "harness/skills/beta/SKILL.md",
    ] {
        assert!(bundles.contains(&format!("# FILE: {path}")));
        assert!(bundles.contains(&format!("# END FILE: {path}")));
    };
    Ok(())
}
#[test]
fn generated_skill_index_and_operating_system_metadata_are_excluded()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository("harness/skills/README.md", "generated index\n")?;
    fixture.write_repository("harness/.DS_Store", "noise\n")?;
    generate(&fixture)?;
    let manifest = fs::read_to_string(fixture.repository().join(".harness-export/00-MANIFEST.md"))?;
    assert!(!manifest.contains("harness/skills/README.md"));
    assert!(!manifest.contains("harness/.DS_Store"));
    Ok(())
}
#[test]
fn check_detects_modified_added_and_disappeared_sources_without_writing()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for mutation in 0..3 {
        let fixture = configured_fixture()?;
        generate(&fixture)?;
        match mutation {
            0 => fixture.write_repository("harness/AGENTS.md", "modified\n")?,
            1 => fixture.write_repository("harness/new/source.txt", "new\n")?,
            _ => {
                fs::remove_file(fixture.repository().join("harness/skills/beta/SKILL.md"))?;
            }
        }
        let before = export_files(&fixture)?;
        let output = arnes(&fixture, &["export", "--check"])?;
        assert!(!output.status.success(), "mutation {mutation}");
        assert_eq!(export_files(&fixture)?, before);
    };
    Ok(())
}
#[test]
fn generation_removes_obsolete_bundles_and_check_only_reports_them()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    generate(&fixture)?;
    let obsolete = fixture.repository().join(".harness-export/99-OBSOLETE.md");
    fs::write(&obsolete, "obsolete\n")?;
    let before = export_files(&fixture)?;
    let check = arnes(&fixture, &["export", "--check"])?;
    assert!(!check.status.success());
    assert_eq!(export_files(&fixture)?, before);
    generate(&fixture)?;
    assert!(!obsolete.exists());
    Ok(())
}
#[test]
fn check_fails_when_export_is_missing_without_writing()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let output = arnes(&fixture, &["export", "--check"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("does not exist"));
    assert!(!fixture.repository().join(".harness-export").exists());
    Ok(())
}
#[test]
fn missing_git_fails_closed_without_writing() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .arg("export")
        .current_dir(fixture.repository())
        .env("HOME", fixture.home())
        .env("PATH", "/nonexistent")
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("Git could not run"));
    assert!(!fixture.repository().join(".harness-export").exists());
    Ok(())
}
