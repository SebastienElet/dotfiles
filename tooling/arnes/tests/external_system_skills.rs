#![cfg(test)]
#[path = "support/skills.rs"]
pub mod skill_support;
pub mod support;
use skill_support::{MANIFEST, configured_fixture, run};
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
fn manifest(skills: &str) -> String {
    MANIFEST . replacen ("resources:" , & format ! ("external:\n  roots:\n    - agent: codex\n      scope: user\n      origin: system\n      location: {{ root: home, path: .codex/skills/.system }}\n  skills:\n{skills}  plugins: []\nresources:") , 1 ,)
}
fn allow(slug: &str) -> String {
    format!("    - agent: codex\n      scope: user\n      origin: system\n      slug: {slug}\n")
}
#[test]
fn codex_system_skills_report_policy_without_claiming_runtime_activation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(&allow("openai-docs")))?;
    fixture.write_home(
        ".codex/skills/.system/openai-docs/SKILL.md",
        "[missing](references/missing.md)\n",
    )?;
    fixture.write_home(".codex/skills/.system/surprise/SKILL.md", "surprise\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains("codex user system skills"));
    assert!(stdout.contains("openai-docs · enabled · healthy · allowed"));
    assert!(stdout.contains("surprise · enabled · healthy · unexpected"));
    assert!(!stdout.contains("local resource"));
    Ok(())
}
#[test]
fn disabled_system_skill_is_visible_but_not_active_or_drift()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(""))?;
    fixture.write_home(".codex/skills/.system/disabled/SKILL.md", "disabled\n")?;
    fixture . write_home (".codex/config.toml" , & format ! ("[[skills.config]]\npath = \"{}/.codex/skills/.system/disabled/SKILL.md\"\nenabled = false\n" , fixture . home () . display ()) ,) ? ;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("disabled · disabled · healthy · unexpected"));
    Ok(())
}
#[test]
fn absent_roots_and_allowed_skills_do_not_drift()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(&allow("openai-docs")))?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("system skills root ~/.codex/skills/.system"));
    assert!(stdout.contains("exposure=absent topology=healthy"));
    assert!(!stdout.contains("skill openai-docs"));
    Ok(())
}
#[test]
fn undeclared_system_root_is_silent_rather_than_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("system skills"), "{stdout}");
    assert!(!stdout.contains("unsupported capabilities"), "{stdout}");
    Ok(())
}
#[test]
fn external_inventory_runs_without_a_managed_projection()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    fixture . write_home (".arnes.yaml" , "version: 1\nagents:\n  - id: codex\n    scopes: [user]\nskills: []\nexternal:\n  roots:\n    - { agent: codex, scope: user, origin: system, location: { root: home, path: .codex/skills/.system } }\n  plugins: []\n  skills:\n    - { agent: codex, scope: user, origin: system, slug: openai-docs }\nresources: []\n" ,) ? ;
    fixture.write_home(".codex/skills/.system/openai-docs/SKILL.md", "docs\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "skills", "--agent", "codex", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("skill projection is not declared or supported"));
    assert!(stdout.contains("openai-docs · enabled · healthy"));
    assert!(stdout.contains("· allowed"));
    Ok(())
}
#[test]
fn valid_absolute_and_relative_links_remain_inside_the_declared_root()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(""))?;
    for slug in ["absolute-target", "relative-target"] {
        fixture.write_home(format!(".codex/skills/.system/{slug}/SKILL.md"), "skill\n")?;
    }
    let root = fixture.home().join(".codex/skills/.system");
    symlink(root.join("absolute-target"), root.join("absolute"))?;
    symlink("relative-target", root.join("relative"))?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 1, "{stdout}");
    let _: () = for slug in ["absolute", "relative"] {
        let line = stdout
            .lines()
            .find(|line| line.contains(&format!("{slug} · enabled")))
            .ok_or("required test value is missing")?;
        assert!(line.contains("· healthy ·"), "{line}");
    };
    Ok(())
}
#[test]
fn dangling_and_intermediate_escape_links_are_broken_without_traversal()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(""))?;
    fs::create_dir_all(fixture.home().join(".codex/skills/.system"))?;
    symlink(
        "missing",
        fixture.home().join(".codex/skills/.system/dangling"),
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("dangling · enabled · broken"));
    assert!(stdout.contains("· broken ·"));
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(""))?;
    let outside = fixture
        .home()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-codex");
    fs::create_dir_all(outside.join("skills/.system/hidden"))?;
    fs::write(outside.join("skills/.system/hidden/SKILL.md"), "hidden\n")?;
    symlink("../outside-codex", fixture.home().join(".codex"))?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("root resolves outside its scope"));
    assert!(!stdout.contains("skill hidden"));
    Ok(())
}
#[test]
fn invalid_and_unreadable_system_roots_are_errors_and_read_only()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(""))?;
    fixture.write_home(".codex/skills/.system", "not a directory\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "skills", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("root is not a directory"));
    let fixture = configured_fixture()?;
    fixture.write_home(".arnes.yaml", &manifest(""))?;
    let root = fixture.home().join(".codex/skills/.system");
    fs::create_dir_all(&root)?;
    let before = fixture.snapshot()?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o000))?;
    let output = fixture.command(["doctor", "skills", "--agent", "codex", "--scope", "user"])?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o755))?;
    assert_eq!(fixture.snapshot()?, before);
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8(output.stdout)?.contains("topology=unreadable"));
    Ok(())
}
