#[path = "support/instructions.rs"]
pub mod instruction_support;
pub mod support;

use instruction_support::{configured_fixture, remove, run};
use std::fs;
use std::os::unix::fs::symlink;
use std::path::Path;

#[test]
fn relative_symlinks_to_expected_sources_are_healthy() {
    let fixture = configured_fixture();
    for (source, destination) in [
        ("AGENTS.md", "CLAUDE.md"),
        ("SOUL.md", "SOUL.md"),
        ("USER.md", "USER.md"),
    ] {
        let destination = fixture.home().join(".claude").join(destination);
        remove(&destination);
        symlink(
            Path::new("../../repository/harness").join(source),
            destination,
        )
        .unwrap();
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
        ],
    );

    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.matches("healthy instructions:").count(), 3);
}

#[test]
fn source_paths_cannot_escape_through_intermediate_symlinks() {
    let fixture = configured_fixture();
    let outside = fixture.repository().parent().unwrap().join("outside");
    fs::create_dir(&outside).unwrap();
    fs::rename(
        fixture.repository().join("harness"),
        outside.join("harness"),
    )
    .unwrap();
    symlink("../outside/harness", fixture.repository().join("harness")).unwrap();
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
    );

    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside the repository"));
}

#[test]
fn destination_paths_cannot_escape_through_intermediate_symlinks() {
    let fixture = configured_fixture();
    let outside = fixture.home().parent().unwrap().join("outside-codex");
    fs::rename(fixture.home().join(".codex"), &outside).unwrap();
    symlink("../outside-codex", fixture.home().join(".codex")).unwrap();
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
    );

    assert_eq!(code, 2);
    assert!(stdout.contains("resolves outside its declared root"));
}

#[test]
fn markdown_code_does_not_create_includes() {
    let fixture = configured_fixture();
    fixture.write_repository(
        "harness/AGENTS.md",
        "~~~text\n```\n@FENCED.md\n```\n~~~\n`@INLINE.md`\n``@DOUBLE.md``\n@SOUL.md\n@USER.md\nrules\n",
    );
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
    );

    assert_eq!(code, 0, "{stdout}");
    assert_eq!(stdout.matches("healthy instructions:").count(), 3);
}

#[test]
fn directory_symlink_aliases_cannot_mask_cycles() {
    let fixture = configured_fixture();
    symlink(".", fixture.repository().join("loop")).unwrap();
    fixture.write_repository("CLAUDE.md", "@loop/CLAUDE.md\n");
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
    );

    assert_eq!(code, 2);
    assert!(stdout.contains("include cycle"));
}
