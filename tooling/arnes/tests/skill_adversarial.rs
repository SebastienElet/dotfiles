#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;

use skill_support::{configured_fixture, link_home_relative, run};
use std::fs;
use std::os::unix::fs::symlink;

#[test]
fn relative_symlinks_to_declared_sources_are_managed() {
    let fixture = configured_fixture();
    let link = fixture.home().join(".claude/skills/alpha");
    fs::remove_file(&link).unwrap();
    link_home_relative(&fixture, "harness/skills/alpha", ".claude/skills/alpha");

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "claude", "--scope", "user", "-v",
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("HEALTHY alpha"));
}

#[test]
fn source_components_cannot_escape_the_repository() {
    let fixture = configured_fixture();
    let source = fixture.repository().join("harness/skills/alpha");
    let outside = fixture.repository().parent().unwrap().join("outside-alpha");
    fs::rename(&source, &outside).unwrap();
    symlink("../../../outside-alpha", source).unwrap();

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "user", "-v",
        ],
    );

    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside the repository"));
}

#[test]
fn destination_components_cannot_escape_home_or_trigger_outside_discovery() {
    let fixture = configured_fixture();
    let outside = fixture.home().parent().unwrap().join("outside-claude");
    fs::create_dir_all(outside.join("skills/plugin-owned")).unwrap();
    fs::remove_dir_all(fixture.home().join(".claude")).unwrap();
    symlink("../outside-claude", fixture.home().join(".claude")).unwrap();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 2);
    assert!(stdout.contains("escapes its scope root"));
    assert!(!stdout.contains("plugin-owned"));
}

#[test]
fn referenced_resource_symlinks_cannot_escape_the_effective_skill() {
    let fixture = configured_fixture();
    let references = fixture.repository().join("harness/skills/alpha/references");
    fs::remove_dir_all(&references).unwrap();
    let outside = fixture.repository().join("outside-references");
    fs::create_dir(&outside).unwrap();
    fs::write(outside.join("guide.md"), "outside\n").unwrap();
    symlink("../../../outside-references", references).unwrap();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    );

    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside its skill"));
}

#[test]
fn non_local_markdown_targets_do_not_create_resource_findings() {
    let fixture = configured_fixture();
    fixture.write_repository("harness/skills/alpha/guide(with-parentheses).md", "guide\n");
    fixture.write_repository(
        "harness/skills/alpha/SKILL.md",
        "[web](https://example.com/missing.md) [anchor](#missing) \
         [absolute](/missing.md) [local](guide(with-parentheses).md) plain-missing.md\n\
         ```sh\nscripts/fenced-example.sh\n```\n",
    );

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("local resource"));
}

#[test]
fn undeclared_aliases_of_managed_sources_remain_unmanaged() {
    let fixture = configured_fixture();
    let destination = fixture.home().join(".cursor/skills/plugin-owned");
    symlink(
        fixture.repository().join("harness/skills/alpha"),
        &destination,
    )
    .unwrap();

    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "cursor", "--scope", "user", "-v",
        ],
    );

    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("plugin-owned · managed · enabled · healthy · unexpected"));
    assert_eq!(stdout.matches("HEALTHY alpha").count(), 1);
}

#[test]
fn expected_skill_roots_must_be_directories() {
    let fixture = configured_fixture();
    fs::remove_file(fixture.home().join(".claude/skills/alpha")).unwrap();
    fs::remove_dir(fixture.home().join(".claude/skills")).unwrap();
    fs::write(fixture.home().join(".claude/skills"), "not a directory").unwrap();

    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "claude", "--scope", "user"],
    );

    assert_eq!(code, 2);
    assert!(stdout.contains("could not be read"));
}
