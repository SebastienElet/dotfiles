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
fn absolute_and_relative_source_and_include_symlinks_are_healthy_inside_repository() {
    for absolute in [false, true] {
        let fixture = configured_fixture();
        let prompt_root = fixture.repository().join("harness/prompts");
        let source = prompt_root.join("deploy.md");
        let source_target = prompt_root.join("internal/deploy.md");
        let include = prompt_root.join("fragments/context.md");
        let include_target = prompt_root.join("fragments/internal/context.md");
        fs::create_dir(prompt_root.join("internal")).unwrap();
        fs::create_dir(prompt_root.join("fragments/internal")).unwrap();
        fs::rename(&source, &source_target).unwrap();
        fs::rename(&include, &include_target).unwrap();
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
        symlink(source_link, source).unwrap();
        symlink(include_link, include).unwrap();

        let (code, stdout, _) = run(&fixture, CLAUDE_USER);

        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("\"state\": \"healthy\""));
    }
}

#[test]
fn lexical_include_escape_is_rejected_without_reading_external_content() {
    let fixture = configured_fixture();
    let outside = fixture.repository().parent().unwrap().join("outside.md");
    fs::write(&outside, "EXTERNAL_SENTINEL\n").unwrap();
    fixture.write_repository(
        "harness/prompts/deploy.md",
        "@../../../outside.md\nDeploy $environment for ${ticket}\n",
    );

    let (code, stdout, _) = run(&fixture, CLAUDE_USER);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("escapes the repository"));
    assert!(!stdout.contains("EXTERNAL_SENTINEL"));
    assert_eq!(fs::read_to_string(outside).unwrap(), "EXTERNAL_SENTINEL\n");
}

#[test]
fn rendered_prompts_do_not_read_import_examples_inside_markdown_fences() {
    let fixture = configured_fixture();
    fixture.write_home(
        ".arnes.yaml",
        &manifest(
            "  - id: fenced\n    source: { root: repository, path: harness/prompts/fenced.md }\n    includes: []\n    variables: [environment, ticket]\n    projections:\n      - agent: claude\n        scope: user\n        representation: rendered\n        destination: { root: home, path: .claude/commands/fenced.md }\n",
        ),
    );
    let source = "```text\n@unmanaged-secret.md\n```\nDeploy $environment for ${ticket}\n";
    fixture.write_repository("harness/prompts/fenced.md", source);
    fixture.write_repository(
        "harness/prompts/unmanaged-secret.md",
        "UNMANAGED_SENTINEL\n",
    );
    fixture.write_home(".claude/commands/fenced.md", source);

    let (code, stdout, _) = run(&fixture, CLAUDE_USER);

    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("\"state\": \"healthy\""));
    assert!(!stdout.contains("UNMANAGED_SENTINEL"));
}

#[test]
fn include_escape_through_an_intermediate_symlink_is_not_traversed() {
    let fixture = configured_fixture();
    let fragments = fixture.repository().join("harness/prompts/fragments");
    let outside = fixture
        .repository()
        .parent()
        .unwrap()
        .join("outside-fragments");
    fs::rename(&fragments, &outside).unwrap();
    fs::write(outside.join("sentinel"), "EXTERNAL_SENTINEL\n").unwrap();
    symlink("../../../outside-fragments", fragments).unwrap();

    let (code, stdout, _) = run(&fixture, CLAUDE_USER);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolves outside the repository"));
    assert!(!stdout.contains("EXTERNAL_SENTINEL"));
    assert_eq!(
        fs::read_to_string(outside.join("sentinel")).unwrap(),
        "EXTERNAL_SENTINEL\n"
    );
}

#[test]
fn source_path_that_leaves_and_reenters_repository_is_rejected() {
    let fixture = configured_fixture();
    let prompts = fixture.repository().join("harness/prompts");
    let outside = fixture
        .repository()
        .parent()
        .unwrap()
        .join("outside-prompts");
    let safe = fixture.repository().join("safe");
    fs::remove_dir_all(&prompts).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::create_dir(&safe).unwrap();
    fs::write(safe.join("deploy.md"), "SAFE_FINAL_TARGET\n").unwrap();
    symlink(safe.join("deploy.md"), outside.join("deploy.md")).unwrap();
    symlink(&outside, &prompts).unwrap();

    let (code, stdout, _) = run(&fixture, CLAUDE_PROJECT);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolves outside the repository"));
    assert!(!stdout.contains("SAFE_FINAL_TARGET"));
}

#[test]
fn final_source_symlinks_cannot_leave_and_reenter_repository() {
    for absolute in [false, true] {
        let fixture = configured_fixture();
        let source = fixture.repository().join("harness/prompts/deploy.md");
        let safe = fixture.repository().join("safe");
        let outside = fixture.repository().parent().unwrap().join("outside-route");
        fs::remove_file(&source).unwrap();
        fs::create_dir(&safe).unwrap();
        fs::create_dir(&outside).unwrap();
        fs::write(safe.join("deploy.md"), "SAFE_FINAL_TARGET\n").unwrap();
        symlink(&safe, outside.join("back")).unwrap();
        let target = if absolute {
            outside.join("back/deploy.md")
        } else {
            PathBuf::from("../../../outside-route/back/deploy.md")
        };
        symlink(target, source).unwrap();

        let (code, stdout, _) = run(&fixture, CLAUDE_PROJECT);

        assert_eq!(code, 2, "{stdout}");
        assert!(stdout.contains("resolves outside the repository"));
        assert!(!stdout.contains("SAFE_FINAL_TARGET"));
    }
}

#[test]
fn include_path_that_leaves_and_reenters_repository_is_rejected() {
    let fixture = configured_fixture();
    let prompt_root = fixture.repository().join("harness/prompts");
    let fragments = prompt_root.join("fragments");
    let outside = fixture
        .repository()
        .parent()
        .unwrap()
        .join("outside-fragments");
    fs::remove_dir_all(&fragments).unwrap();
    fs::create_dir(&outside).unwrap();
    fs::create_dir(prompt_root.join("safe")).unwrap();
    fs::write(prompt_root.join("safe/context.md"), "SAFE_FINAL_TARGET\n").unwrap();
    symlink(
        prompt_root.join("safe/context.md"),
        outside.join("context.md"),
    )
    .unwrap();
    symlink(&outside, &fragments).unwrap();

    let (code, stdout, _) = run(&fixture, CLAUDE_USER);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("resolves outside the repository"));
    assert!(!stdout.contains("SAFE_FINAL_TARGET"));
}

#[test]
fn absolute_and_relative_source_symlinks_outside_repository_fail_closed() {
    for absolute in [false, true] {
        let fixture = configured_fixture();
        let source = fixture.repository().join("harness/prompts/deploy.md");
        let outside = fixture.repository().parent().unwrap().join("outside.md");
        fs::rename(&source, &outside).unwrap();
        let target = if absolute {
            outside
        } else {
            PathBuf::from("../../../outside.md")
        };
        symlink(target, source).unwrap();

        let (code, stdout, _) = run(&fixture, CLAUDE_PROJECT);

        assert_eq!(code, 2, "{stdout}");
        assert!(stdout.contains("resolves outside the repository"));
    }
}
