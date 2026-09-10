#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;
use skill_support::{configured_fixture, link_home_relative, run};
use std::fs;
use std::os::unix::fs::symlink;
#[test]
fn relative_symlinks_to_declared_sources_are_managed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let link = fixture.home().join(".claude/skills/alpha");
    fs::remove_file(&link)?;
    link_home_relative(&fixture, "harness/skills/alpha", ".claude/skills/alpha")?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "claude", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("HEALTHY alpha"));
    Ok(())
}
#[test]
fn source_components_cannot_escape_the_repository()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let source = fixture.repository().join("harness/skills/alpha");
    let outside = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-alpha");
    fs::rename(&source, &outside)?;
    symlink("../../../outside-alpha", source)?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside the repository"));
    Ok(())
}
#[test]
fn destination_components_cannot_escape_home_or_trigger_outside_discovery()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let outside = fixture
        .home()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-claude");
    fs::create_dir_all(outside.join("skills/plugin-owned"))?;
    fs::remove_dir_all(fixture.home().join(".claude"))?;
    symlink("../outside-claude", fixture.home().join(".claude"))?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("escapes its scope root"));
    assert!(!stdout.contains("plugin-owned"));
    Ok(())
}
#[test]
fn referenced_resource_symlinks_cannot_escape_the_effective_skill()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let references = fixture.repository().join("harness/skills/alpha/references");
    fs::remove_dir_all(&references)?;
    let outside = fixture.repository().join("outside-references");
    fs::create_dir(&outside)?;
    fs::write(outside.join("guide.md"), "outside\n")?;
    symlink("../../../outside-references", references)?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside its skill"));
    Ok(())
}
#[test]
fn non_local_markdown_targets_do_not_create_resource_findings()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository("harness/skills/alpha/guide(with-parentheses).md", "guide\n")?;
    fixture.write_repository(
        "harness/skills/alpha/SKILL.md",
        "[web](https://example.com/missing.md) [anchor](#missing) \
         [absolute](/missing.md) [local](guide(with-parentheses).md) plain-missing.md\n\
         ```sh\nscripts/fenced-example.sh\n```\n",
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("local resource"));
    Ok(())
}
#[test]
fn undeclared_aliases_of_managed_sources_remain_unmanaged()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let destination = fixture.home().join(".cursor/skills/plugin-owned");
    symlink(
        fixture.repository().join("harness/skills/alpha"),
        &destination,
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("plugin-owned · managed · enabled · healthy · unexpected"));
    assert_eq!(stdout.matches("HEALTHY alpha").count(), 1);
    Ok(())
}
#[test]
fn expected_skill_roots_must_be_directories() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    fs::remove_file(fixture.home().join(".claude/skills/alpha"))?;
    fs::remove_dir(fixture.home().join(".claude/skills"))?;
    fs::write(fixture.home().join(".claude/skills"), "not a directory")?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("could not be read"));
    Ok(())
}
