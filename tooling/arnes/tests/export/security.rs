use super::{arnes, configured_fixture, export_files, generate};
use std::fs;
use std::os::unix::fs::symlink;
#[test]
fn obvious_sensitive_paths_fail_closed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for path in [
        "harness/skills/alpha/.env",
        "harness/skills/alpha/credentials.yaml",
        "harness/skills/alpha/prod-credentials.yml",
        "harness/skills/alpha/secret.yaml",
        "harness/skills/alpha/api_key.yaml",
    ] {
        let fixture = configured_fixture()?;
        fixture.write_repository(path, "api_token: super-secret-value\n")?;
        let output = arnes(&fixture, &["export"])?;
        assert!(!output.status.success(), "{path}");
        assert!(String::from_utf8_lossy(&output.stderr).contains("sensitive"));
        assert!(!fixture.repository().join(".harness-export").exists());
    };
    Ok(())
}
#[test]
fn selected_links_cannot_reintroduce_ignored_sources()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository(
        ".gitignore",
        ".harness-export/\nharness/skills/alpha/.claude-flow/\n",
    )?;
    fixture.write_repository(
        "harness/skills/alpha/.claude-flow/credentials.yaml",
        "api_key: super-secret-value\n",
    )?;
    symlink(
        ".claude-flow/credentials.yaml",
        fixture.repository().join("harness/skills/alpha/public.md"),
    )?;
    let output = arnes(&fixture, &["export"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unselected target"));
    Ok(())
}
#[test]
fn ignored_relevant_sources_fail_closed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository(".gitignore", ".harness-export/\nharness/skills/ignored/\n")?;
    fixture.write_repository("harness/skills/ignored/SKILL.md", "ignored\n")?;
    let output = arnes(&fixture, &["export"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ignored harness source"));
    Ok(())
}
#[test]
fn external_symlinks_and_hardlinks_fail_closed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for hardlink in [false, true] {
        let fixture = configured_fixture()?;
        let selected = fixture.repository().join("harness/skills/alpha/SKILL.md");
        let outside = fixture.home().join("sensitive");
        fs::write(&outside, "sensitive\n")?;
        fs::remove_file(&selected)?;
        if hardlink {
            fs::hard_link(&outside, &selected)?;
        } else {
            symlink(&outside, &selected)?;
        }
        let output = arnes(&fixture, &["export"])?;
        assert!(!output.status.success());
        assert!(String::from_utf8_lossy(&output.stderr).contains("refusing"));
    };
    Ok(())
}
#[test]
fn external_parent_links_cannot_publish_sources()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (parent, selected) in [
        ("harness", "harness/AGENTS.md"),
        ("harness/skills/alpha", "harness/skills/alpha/SKILL.md"),
        ("home", "home/.arnes.yaml"),
    ] {
        let fixture = configured_fixture()?;
        let existing = if parent == "harness" {
            None
        } else {
            generate(&fixture)?;
            Some(export_files(&fixture)?)
        };
        let outside = fixture.home().join("external-parent");
        fs::rename(fixture.repository().join(parent), &outside)?;
        fs::write(
            outside.join(selected.rsplit('/').next().ok_or("missing name")?),
            "outside-source-marker\n",
        )?;
        if parent == "harness/skills/alpha" {
            symlink(
                "../../../home/external-parent",
                fixture.repository().join(parent),
            )?;
        } else {
            symlink(&outside, fixture.repository().join(parent))?;
        }

        let output = arnes(&fixture, &["export"])?;
        assert!(!output.status.success(), "{parent}");
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("refusing"),
            "{parent}"
        );
        match existing {
            Some(before) => assert_eq!(export_files(&fixture)?, before, "{parent}"),
            None => assert!(!fixture.repository().join(".harness-export").exists()),
        }
    }
    Ok(())
}
#[test]
fn unreadable_source_parent_fails_before_snapshot_creation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let parent = fixture.repository().join("home");
    fs::remove_dir_all(&parent)?;
    symlink(fixture.home().join("missing-parent"), &parent)?;

    let output = arnes(&fixture, &["export"])?;
    assert!(!output.status.success());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("parent is unreadable"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!fixture.repository().join(".harness-export").exists());
    Ok(())
}
#[test]
fn export_path_symlinks_and_regular_files_are_refused_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for symlink_path in [false, true] {
        let fixture = configured_fixture()?;
        let export = fixture.repository().join(".harness-export");
        let sentinel = fixture.home().join("sentinel");
        fs::create_dir(&sentinel)?;
        fs::write(sentinel.join("value"), "unchanged\n")?;
        if symlink_path {
            symlink(&sentinel, &export)?;
        } else {
            fs::write(&export, "unchanged\n")?;
        }
        let output = arnes(&fixture, &["export"])?;
        assert!(!output.status.success());
        assert_eq!(fs::read(sentinel.join("value"))?, b"unchanged\n");
        if !symlink_path {
            assert_eq!(fs::read(export)?, b"unchanged\n");
        }
    };
    Ok(())
}
