#![cfg(test)]
#[path = "support/instructions.rs"]
pub mod instruction_support;
pub mod support;
use instruction_support::{configured_fixture, remove, run};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::{Path, PathBuf};
#[test]
fn relative_symlinks_to_expected_sources_are_healthy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    for (source, destination) in [
        ("AGENTS.md", "CLAUDE.md"),
        ("SOUL.md", "SOUL.md"),
        ("USER.md", "USER.md"),
    ] {
        let destination = fixture.home().join(".claude").join(destination);
        remove(&destination)?;
        symlink(
            Path::new("../../repository/harness").join(source),
            destination,
        )?;
    }
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
            "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.matches("healthy instructions:").count(), 3);
    Ok(())
}
#[test]
fn source_and_include_symlinks_within_the_repository_are_healthy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for absolute in [false, true] {
        let fixture = configured_fixture()?;
        let linked = fixture.repository().join("harness/linked");
        fs::create_dir(&linked)?;
        for name in ["AGENTS.md", "SOUL.md"] {
            let source = fixture.repository().join("harness").join(name);
            let target = linked.join(name);
            fs::rename(&source, &target)?;
            let link_target = if absolute {
                target
            } else {
                PathBuf::from("linked").join(name)
            };
            symlink(link_target, source)?;
        }
        let (code, stdout, _) = run(
            &fixture,
            &[
                "doctor",
                "instructions",
                "--agent",
                "claude",
                "--scope",
                "user",
                "-v",
            ],
        )?;
        assert_eq!(code, 0, "{stdout}");
        assert_eq!(stdout.matches("healthy instructions:").count(), 3);
    };
    Ok(())
}
#[test]
fn dangling_source_includes_fail_closed() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let include = fixture.repository().join("harness/SOUL.md");
    remove(&include)?;
    symlink("missing.md", include)?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "codex",
            "--scope",
            "user",
            "-v",
        ],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("include"));
    assert!(stdout.contains("SOUL.md is missing"));
    Ok(())
}
#[test]
fn source_symlinks_outside_the_repository_fail_closed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for absolute in [false, true] {
        let fixture = configured_fixture()?;
        let source = fixture.repository().join("harness/AGENTS.md");
        let outside = fixture
            .repository()
            .parent()
            .ok_or("fixture path has no parent")?
            .join("outside.md");
        fs::rename(&source, &outside)?;
        let target = if absolute {
            outside
        } else {
            PathBuf::from("../../outside.md")
        };
        symlink(target, source)?;
        let (code, stdout, _) = run(
            &fixture,
            &[
                "doctor",
                "instructions",
                "--agent",
                "claude",
                "--scope",
                "user",
            ],
        )?;
        assert_eq!(code, 2);
        assert!(stdout.contains("resolves outside the repository"));
    };
    Ok(())
}
#[test]
fn source_paths_cannot_escape_through_intermediate_symlinks()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let outside = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside");
    fs::create_dir(&outside)?;
    fs::rename(
        fixture.repository().join("harness"),
        outside.join("harness"),
    )?;
    symlink("../outside/harness", fixture.repository().join("harness"))?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
        ],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside the repository"));
    Ok(())
}
#[test]
fn destination_paths_cannot_escape_through_intermediate_symlinks()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let outside = fixture
        .home()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-codex");
    fs::rename(fixture.home().join(".codex"), &outside)?;
    symlink("../outside-codex", fixture.home().join(".codex"))?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "codex",
            "--scope",
            "user",
        ],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside its declared root"));
    Ok(())
}
#[test]
fn markdown_code_does_not_create_includes() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    fixture . write_repository ("harness/AGENTS.md" , "~~~text\n```\n@FENCED.md\n```\n~~~\n`@INLINE.md`\n``@DOUBLE.md``\n@SOUL.md\n@USER.md\nrules\n" ,) ? ;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "user",
            "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.matches("healthy instructions:").count(), 3);
    Ok(())
}
#[test]
fn directory_symlink_aliases_cannot_mask_cycles()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    symlink(".", fixture.repository().join("loop"))?;
    fixture.write_repository("CLAUDE.md", "@loop/CLAUDE.md\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor",
            "instructions",
            "--agent",
            "claude",
            "--scope",
            "project",
        ],
    )?;
    assert_eq!(code, 2);
    assert!(stdout.contains("include cycle"));
    Ok(())
}
