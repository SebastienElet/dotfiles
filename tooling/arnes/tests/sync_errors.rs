#![cfg(test)]
pub mod support;

use serde_json::Value;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use std::process::Command;
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user]
  - id: codex
    scopes: [user]
skills:
  - slug: alpha
    installations:
      - { agent: claude, scope: user }
      - { agent: codex, scope: user }
resources:
  - id: claude-skills
    kind: skills
    agent: claude
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .claude/skills }
  - id: codex-skills
    kind: skills
    agent: codex
    scope: user
    layout: leaves
    source: { root: repository, path: harness/skills }
    destination: { root: home, path: .agents/skills }
  - id: rule
    kind: rules
    agent: claude
    scope: user
    source: { root: repository, path: harness/rules/rule.md }
    destination: { root: home, path: .claude/rules/rule.md }
";

fn fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    fixture.write_repository("harness/skills/alpha/SKILL.md", "alpha\n")?;
    fixture.write_repository("harness/rules/rule.md", "rule\n")?;
    Ok(fixture)
}

fn json(
    fixture: &Fixture,
    resource: &str,
    agent: &str,
) -> Result<(i32, Value), Box<dyn std::error::Error + Send + Sync>> {
    let output = fixture.command([
        "sync", resource, "--agent", agent, "--scope", "user", "--format", "json",
    ])?;
    Ok((
        output.status.code().ok_or("status")?,
        serde_json::from_slice(&output.stdout)?,
    ))
}

#[test]
fn json_reports_scope_applied_current_and_refused_without_false_success() -> TestResult {
    let fixture = fixture()?;
    let (code, report) = json(&fixture, "rules", "claude")?;
    assert_eq!(code, 0);
    assert_eq!(report.get("resource").ok_or("resource")?, "rules");
    assert_eq!(report.get("agent").ok_or("agent")?, "claude");
    assert_eq!(report.get("scope").ok_or("scope")?, "user");
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "applied"
    );
    let (code, report) = json(&fixture, "rules", "claude")?;
    assert_eq!(code, 0);
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "current"
    );
    fs::remove_file(fixture.home().join(".claude/rules/rule.md"))?;
    fixture.write_home(".claude/rules/rule.md", "local")?;
    let before = fixture.snapshot()?;
    let (code, report) = json(&fixture, "rules", "claude")?;
    assert_eq!(code, 1);
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "refused"
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn source_aliases_and_wrong_types_are_refused_before_creating_any_destination() -> TestResult {
    for kind in [
        "outside-link",
        "directory",
        "non-utf8",
        "hardlink",
        "reference-outside",
    ] {
        let fixture = fixture()?;
        let outside = tempfile::tempdir()?;
        let source = fixture.repository().join("harness/rules/rule.md");
        let resource = if kind == "reference-outside" {
            "skills"
        } else {
            "rules"
        };
        match kind {
            "outside-link" => {
                fs::remove_file(&source)?;
                fs::write(outside.path().join("rule.md"), "external")?;
                symlink(outside.path().join("rule.md"), &source)?;
            }
            "directory" => {
                fs::remove_file(&source)?;
                fs::create_dir(&source)?;
            }
            "non-utf8" => fs::write(&source, [0xff])?,
            "hardlink" => fs::hard_link(&source, outside.path().join("alias"))?,
            "reference-outside" => {
                fixture.write_repository("harness/skills/alpha/SKILL.md", "[outside](guide.md)")?;
                fs::write(outside.path().join("guide.md"), "outside")?;
                symlink(
                    outside.path().join("guide.md"),
                    fixture.repository().join("harness/skills/alpha/guide.md"),
                )?;
            }
            _ => return Err("unknown fixture".into()),
        }
        let before = fixture.snapshot()?;
        let (code, report) = json(&fixture, resource, "claude")?;
        assert_eq!(code, 1);
        assert_eq!(
            report.pointer("/entries/0/state").ok_or("state")?,
            "refused"
        );
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn unavailable_external_inventory_and_retired_skills_do_not_authorize_mutation() -> TestResult {
    let fixture = fixture()?;
    fixture.write_home(".claude/skills/retired/SKILL.md", "retired")?;
    fixture.write_home(
        ".codex/config.toml",
        "[plugins.\"external@marketplace\"]\nenabled = true\n",
    )?;
    fixture.write_home(
        ".codex/plugins/cache/marketplace/external/one/manifest.json",
        "unknown",
    )?;
    fixture.write_home(
        ".codex/plugins/cache/marketplace/external/two/manifest.json",
        "unknown",
    )?;
    let before = fixture.snapshot()?;
    let (code, _) = json(&fixture, "skills", "codex")?;
    assert_eq!(code, 0);
    let (code, _) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 0);
    let after = fixture.snapshot()?;
    for (path, entry) in before {
        assert_eq!(after.get(&path), Some(&entry), "{}", path.display());
    }
    assert!(
        fixture
            .home()
            .join(".claude/skills/retired/SKILL.md")
            .exists()
    );
    Ok(())
}

#[test]
fn source_validation_failure_preserves_other_selected_projections() -> TestResult {
    let fixture = fixture()?;
    let manifest = MANIFEST.replace("  - slug: alpha\n", "  - slug: missing\n    installations:\n      - { agent: claude, scope: user }\n  - slug: alpha\n");
    fixture.write_home(".arnes.yaml", &manifest)?;
    let before = fixture.snapshot()?;
    let (code, report) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 1);
    assert_eq!(
        report
            .get("entries")
            .and_then(Value::as_array)
            .ok_or("entries")?
            .len(),
        2
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn native_write_failure_is_explicit_and_preserves_existing_state() -> TestResult {
    let fixture = fixture()?;
    fixture.write_home(".claude/rules/neighbor.md", "neighbor")?;
    let directory = fixture.home().join(".claude/rules");
    let original = fs::metadata(&directory)?.permissions();
    fs::set_permissions(&directory, fs::Permissions::from_mode(0o500))?;
    let before = fixture.snapshot()?;
    let output = json(&fixture, "rules", "claude");
    let after = fixture.snapshot();
    fs::set_permissions(&directory, original)?;
    let (code, report) = output?;
    assert_eq!(code, 2);
    assert_eq!(report.pointer("/entries/0/state").ok_or("state")?, "failed");
    assert_eq!(after?, before);
    Ok(())
}

#[test]
fn preserves_relative_correct_links_and_their_inode() -> TestResult {
    let fixture = fixture()?;
    let directory = fixture.home().join(".claude/rules");
    fs::create_dir_all(&directory)?;
    let destination = directory.join("rule.md");
    symlink("../../../repository/harness/rules/rule.md", &destination)?;
    let identity = fs::symlink_metadata(&destination)?.ino();
    let before = fixture.snapshot()?;
    let (code, report) = json(&fixture, "rules", "claude")?;
    assert_eq!(code, 0);
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "current"
    );
    assert_eq!(fs::symlink_metadata(destination)?.ino(), identity);
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn malformed_manifest_is_rejected_before_creating_scope_directories() -> TestResult {
    let fixture = fixture()?;
    fixture.write_home(".arnes.yaml", "invalid: [")?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["sync", "rules", "--agent", "claude", "--scope", "user"])?;
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn a_home_inside_the_skill_source_cannot_receive_new_projections() -> TestResult {
    let fixture = fixture()?;
    let home = fixture.repository().join("harness/skills/alpha");
    fs::write(home.join(".arnes.yaml"), MANIFEST)?;
    let before = fixture.snapshot()?;
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args([
            "sync", "skills", "--agent", "claude", "--scope", "user", "--format", "json",
        ])
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", &home)
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    let report: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "refused"
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn user_skills_use_the_deployed_manifest_checkout_instead_of_current_repository() -> TestResult {
    let fixture = fixture()?;
    let deployment = tempfile::tempdir()?;
    fs::create_dir_all(deployment.path().join("home"))?;
    fs::create_dir_all(deployment.path().join("harness/skills/alpha"))?;
    fs::write(deployment.path().join("home/.arnes.yaml"), MANIFEST)?;
    fs::write(
        deployment.path().join("harness/skills/alpha/SKILL.md"),
        "deployed",
    )?;
    fs::remove_file(fixture.home().join(".arnes.yaml"))?;
    symlink(
        deployment.path().join("home/.arnes.yaml"),
        fixture.home().join(".arnes.yaml"),
    )?;
    fs::remove_file(fixture.repository().join("harness/skills/alpha/SKILL.md"))?;
    let (code, report) = json(&fixture, "skills", "codex")?;
    assert_eq!(code, 0);
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "applied"
    );
    assert_eq!(
        fs::canonicalize(fixture.home().join(".agents/skills/alpha"))?,
        fs::canonicalize(deployment.path().join("harness/skills/alpha"))?
    );
    let doctor = fixture.command(["doctor", "skills", "--agent", "codex", "--scope", "user"])?;
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    Ok(())
}

#[test]
fn an_unselected_canonical_skill_cannot_be_used_as_home_for_new_links() -> TestResult {
    let fixture = fixture()?;
    fixture.write_repository("harness/skills/local/SKILL.md", "local")?;
    let home = fixture.repository().join("harness/skills/local");
    fs::write(home.join(".arnes.yaml"), MANIFEST)?;
    let before = fixture.snapshot()?;
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["sync", "skills", "--agent", "claude", "--scope", "user"])
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", &home)
        .output()?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn rules_never_create_destinations_inside_unselected_canonical_sources() -> TestResult {
    for boundary in [
        "harness/skills/local",
        "home/source",
        ".agents/skills/local",
        "user-skills/local",
    ] {
        let fixture = fixture()?;
        fixture.write_repository(std::path::Path::new(boundary).join("source.md"), "source")?;
        if boundary == "user-skills/local" {
            fs::remove_dir_all(fixture.repository().join("harness/skills"))?;
            symlink(
                "../user-skills",
                fixture.repository().join("harness/skills"),
            )?;
        }
        let home = fixture.repository().join(boundary);
        fs::write(home.join(".arnes.yaml"), MANIFEST)?;
        let before = fixture.snapshot()?;
        let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["sync", "rules", "--agent", "claude", "--scope", "user"])
            .current_dir(fixture.repository())
            .env_clear()
            .env("HOME", &home)
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(1),
            "{boundary}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn user_skill_destinations_cannot_add_entries_to_the_project_source_collection() -> TestResult {
    let fixture = fixture()?;
    fixture.write_repository(".agents/skills/project/SKILL.md", "project")?;
    fixture.write_repository(".arnes.yaml", MANIFEST)?;
    let before = fixture.snapshot()?;
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["sync", "skills", "--agent", "codex", "--scope", "user"])
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.repository())
        .output()?;
    assert_eq!(
        output.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&output.stdout)
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn refuses_a_user_skill_aliasing_a_project_skill() -> TestResult {
    let fixture = fixture()?;
    fixture.write_repository(".agents/skills/project/SKILL.md", "project")?;
    fs::remove_dir_all(fixture.repository().join("harness/skills/alpha"))?;
    symlink(
        "../../.agents/skills/project",
        fixture.repository().join("harness/skills/alpha"),
    )?;
    let before = fixture.snapshot()?;
    let (code, report) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 1);
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "refused"
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn refuses_a_user_collection_aliasing_the_project_collection() -> TestResult {
    let fixture = fixture()?;
    fixture.write_repository(".agents/skills/alpha/SKILL.md", "project")?;
    fs::remove_dir_all(fixture.repository().join("harness/skills"))?;
    symlink(
        "../.agents/skills",
        fixture.repository().join("harness/skills"),
    )?;
    let before = fixture.snapshot()?;
    let (code, report) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 1);
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "refused"
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn user_skill_links_within_the_canonical_user_collection_remain_supported() -> TestResult {
    let fixture = fixture()?;
    fs::rename(
        fixture.repository().join("harness/skills/alpha"),
        fixture.repository().join("harness/skills/user-alpha"),
    )?;
    symlink(
        "user-alpha",
        fixture.repository().join("harness/skills/alpha"),
    )?;
    let (code, _) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 0);
    assert_eq!(
        fs::canonicalize(fixture.home().join(".claude/skills/alpha"))?,
        fs::canonicalize(fixture.repository().join("harness/skills/user-alpha"))?
    );
    let before = fixture.snapshot()?;
    let (code, report) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 0);
    assert_eq!(
        report.pointer("/entries/0/state").ok_or("state")?,
        "current"
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn user_collection_links_to_a_separate_user_directory_remain_supported() -> TestResult {
    let fixture = fixture()?;
    fs::rename(
        fixture.repository().join("harness/skills"),
        fixture.repository().join("user-skills"),
    )?;
    symlink(
        "../user-skills",
        fixture.repository().join("harness/skills"),
    )?;
    let (code, _) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 0);
    assert_eq!(
        fs::canonicalize(fixture.home().join(".claude/skills/alpha"))?,
        fs::canonicalize(fixture.repository().join("user-skills/alpha"))?
    );
    let before = fixture.snapshot()?;
    let (code, _) = json(&fixture, "skills", "claude")?;
    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot()?, before);
    let doctor = fixture.command(["doctor", "skills", "--agent", "claude", "--scope", "user"])?;
    assert!(
        doctor.status.success(),
        "{}",
        String::from_utf8_lossy(&doctor.stdout)
    );
    Ok(())
}
