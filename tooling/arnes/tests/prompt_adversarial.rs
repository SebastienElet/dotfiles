#![cfg(test)]
#[path = "support/prompts.rs"]
pub mod prompt_support;
pub mod support;
use prompt_support::{configured_fixture, manifest, run};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::PathBuf;
const CLAUDE_USER: &[&str] = &[
    "doctor", "prompts", "--agent", "claude", "--scope", "user", "--format", "json",
];
const CLAUDE_PROJECT: &[&str] = &[
    "doctor", "prompts", "--agent", "claude", "--scope", "project", "--format", "json",
];
#[test]
fn absolute_and_relative_source_and_include_symlinks_are_healthy_inside_repository()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for absolute in [false, true] {
        let fixture = configured_fixture()?;
        let prompt_root = fixture.repository().join("harness/prompts");
        let source = prompt_root.join("deploy.md");
        let source_target = prompt_root.join("internal/deploy.md");
        let include = prompt_root.join("fragments/context.md");
        let include_target = prompt_root.join("fragments/internal/context.md");
        fs::create_dir(prompt_root.join("internal"))?;
        fs::create_dir(prompt_root.join("fragments/internal"))?;
        fs::rename(&source, &source_target)?;
        fs::rename(&include, &include_target)?;
        let source_link = if absolute {
            source_target
        } else {
            PathBuf::from("internal/deploy.md")
        };
        let include_link = if absolute {
            include_target
        } else {
            PathBuf::from("internal/context.md")
        };
        symlink(source_link, source)?;
        symlink(include_link, include)?;
        let (code, stdout, _) = run(&fixture, CLAUDE_USER)?;
        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("\"state\": \"healthy\""));
    };
    Ok(())
}
#[test]
fn lexical_include_escape_is_rejected_without_reading_external_content()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let outside = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside.md");
    fs::write(&outside, "EXTERNAL_SENTINEL\n")?;
    fixture.write_repository(
        "harness/prompts/deploy.md",
        "@../../../outside.md\nDeploy $environment for ${ticket}\n",
    )?;
    let (code, stdout, _) = run(&fixture, CLAUDE_USER)?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("escapes the repository"));
    assert!(!stdout.contains("EXTERNAL_SENTINEL"));
    assert_eq!(fs::read_to_string(outside)?, "EXTERNAL_SENTINEL\n");
    Ok(())
}
#[test]
fn rendered_prompts_do_not_read_import_examples_inside_markdown_fences()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture . write_home (".arnes.yaml" , & manifest ("  - id: fenced\n    source: { root: repository, path: harness/prompts/fenced.md }\n    includes: []\n    variables: [environment, ticket]\n    projections:\n      - agent: claude\n        scope: user\n        representation: rendered\n        destination: { root: home, path: .claude/commands/fenced.md }\n" ,) ,) ? ;
    let source = "```text\n@unmanaged-secret.md\n```\nDeploy $environment for ${ticket}\n";
    fixture.write_repository("harness/prompts/fenced.md", source)?;
    fixture.write_repository(
        "harness/prompts/unmanaged-secret.md",
        "UNMANAGED_SENTINEL\n",
    )?;
    fixture.write_home(".claude/commands/fenced.md", source)?;
    let (code, stdout, _) = run(&fixture, CLAUDE_USER)?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("\"state\": \"healthy\""));
    assert!(!stdout.contains("UNMANAGED_SENTINEL"));
    Ok(())
}
#[test]
fn include_escape_through_an_intermediate_symlink_is_not_traversed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let fragments = fixture.repository().join("harness/prompts/fragments");
    let outside = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-fragments");
    fs::rename(&fragments, &outside)?;
    fs::write(outside.join("sentinel"), "EXTERNAL_SENTINEL\n")?;
    symlink("../../../outside-fragments", fragments)?;
    let (code, stdout, _) = run(&fixture, CLAUDE_USER)?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolves outside the repository"));
    assert!(!stdout.contains("EXTERNAL_SENTINEL"));
    assert_eq!(
        fs::read_to_string(outside.join("sentinel"))?,
        "EXTERNAL_SENTINEL\n"
    );
    Ok(())
}
#[test]
fn source_path_that_leaves_and_reenters_repository_is_rejected()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let prompts = fixture.repository().join("harness/prompts");
    let outside = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-prompts");
    let safe = fixture.repository().join("safe");
    fs::remove_dir_all(&prompts)?;
    fs::create_dir(&outside)?;
    fs::create_dir(&safe)?;
    fs::write(safe.join("deploy.md"), "SAFE_FINAL_TARGET\n")?;
    symlink(safe.join("deploy.md"), outside.join("deploy.md"))?;
    symlink(&outside, &prompts)?;
    let (code, stdout, _) = run(&fixture, CLAUDE_PROJECT)?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolves outside the repository"));
    assert!(!stdout.contains("SAFE_FINAL_TARGET"));
    Ok(())
}
#[test]
fn final_source_symlinks_cannot_leave_and_reenter_repository()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for absolute in [false, true] {
        let fixture = configured_fixture()?;
        let source = fixture.repository().join("harness/prompts/deploy.md");
        let safe = fixture.repository().join("safe");
        let outside = fixture
            .repository()
            .parent()
            .ok_or("fixture path has no parent")?
            .join("outside-route");
        fs::remove_file(&source)?;
        fs::create_dir(&safe)?;
        fs::create_dir(&outside)?;
        fs::write(safe.join("deploy.md"), "SAFE_FINAL_TARGET\n")?;
        symlink(&safe, outside.join("back"))?;
        let target = if absolute {
            outside.join("back/deploy.md")
        } else {
            PathBuf::from("../../../outside-route/back/deploy.md")
        };
        symlink(target, source)?;
        let (code, stdout, _) = run(&fixture, CLAUDE_PROJECT)?;
        assert_eq!(code, 2, "{stdout}");
        assert!(stdout.contains("resolves outside the repository"));
        assert!(!stdout.contains("SAFE_FINAL_TARGET"));
    };
    Ok(())
}
#[test]
fn include_path_that_leaves_and_reenters_repository_is_rejected()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let prompt_root = fixture.repository().join("harness/prompts");
    let fragments = prompt_root.join("fragments");
    let outside = fixture
        .repository()
        .parent()
        .ok_or("fixture path has no parent")?
        .join("outside-fragments");
    fs::remove_dir_all(&fragments)?;
    fs::create_dir(&outside)?;
    fs::create_dir(prompt_root.join("safe"))?;
    fs::write(prompt_root.join("safe/context.md"), "SAFE_FINAL_TARGET\n")?;
    symlink(
        prompt_root.join("safe/context.md"),
        outside.join("context.md"),
    )?;
    symlink(&outside, &fragments)?;
    let (code, stdout, _) = run(&fixture, CLAUDE_USER)?;
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolves outside the repository"));
    assert!(!stdout.contains("SAFE_FINAL_TARGET"));
    Ok(())
}
#[test]
fn absolute_and_relative_source_symlinks_outside_repository_fail_closed()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for absolute in [false, true] {
        let fixture = configured_fixture()?;
        let source = fixture.repository().join("harness/prompts/deploy.md");
        let outside = fixture
            .repository()
            .parent()
            .ok_or("fixture path has no parent")?
            .join("outside.md");
        fs::rename(&source, &outside)?;
        let target = if absolute {
            outside
        } else {
            PathBuf::from("../../../outside.md")
        };
        symlink(target, source)?;
        let (code, stdout, _) = run(&fixture, CLAUDE_PROJECT)?;
        assert_eq!(code, 2, "{stdout}");
        assert!(stdout.contains("resolves outside the repository"));
    };
    Ok(())
}
