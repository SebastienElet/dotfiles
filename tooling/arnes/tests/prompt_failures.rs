#[path = "support/prompts.rs"]
pub mod prompt_support;
pub mod support;

use prompt_support::{
    SOURCE, configured_fixture, manifest, project_prompt, run, run_with_unreadable,
};
use std::fs;
use std::os::unix::fs::symlink;
use support::Fixture;

const CLAUDE_PROJECT: &[&str] = &[
    "doctor", "prompts", "--agent", "claude", "--scope", "project", "--format", "json",
];

#[test]
fn missing_wrong_type_and_dangling_sources_fail_closed() {
    let fixture = configured_fixture();
    fs::remove_file(source(&fixture)).unwrap();
    assert_error(&fixture, "source harness/prompts/deploy.md is missing");

    let fixture = configured_fixture();
    fs::remove_file(source(&fixture)).unwrap();
    fs::create_dir(source(&fixture)).unwrap();
    assert_error(
        &fixture,
        "source harness/prompts/deploy.md is not a regular file",
    );

    let fixture = configured_fixture();
    fs::remove_file(source(&fixture)).unwrap();
    symlink("missing.md", source(&fixture)).unwrap();
    assert_error(
        &fixture,
        "source harness/prompts/deploy.md is a dangling symlink",
    );
}

#[test]
fn unreadable_sources_fail_closed_without_mutation() {
    let fixture = configured_fixture();
    let (code, stdout, _) = run_with_unreadable(&fixture, &source(&fixture), CLAUDE_PROJECT);

    assert_eq!(code, 2);
    assert!(stdout.contains("source harness/prompts/deploy.md could not be read"));
}

#[test]
fn missing_stale_and_wrong_type_destinations_are_drift() {
    let fixture = configured_fixture();
    fs::remove_file(destination(&fixture)).unwrap();
    assert_drift(&fixture, "is missing");

    let fixture = configured_fixture();
    fixture.write_repository(".claude/commands/deploy.md", "stale\n");
    assert_drift(&fixture, "is stale");

    let fixture = configured_fixture();
    fs::remove_file(destination(&fixture)).unwrap();
    fs::create_dir(destination(&fixture)).unwrap();
    assert_drift(&fixture, "is not a regular file");
}

#[test]
fn unreadable_destinations_are_errors_without_mutation() {
    let fixture = configured_fixture();
    let (code, stdout, _) = run_with_unreadable(&fixture, &destination(&fixture), CLAUDE_PROJECT);

    assert_eq!(code, 2);
    assert!(stdout.contains("could not be read"));
}

#[test]
fn symlink_destinations_are_drift_without_traversing_their_target() {
    for wrong_target in [false, true] {
        let fixture = configured_fixture();
        fs::remove_file(destination(&fixture)).unwrap();
        let target = if wrong_target {
            context(&fixture)
        } else {
            source(&fixture)
        };
        symlink(target, destination(&fixture)).unwrap();

        let (code, stdout, _) = run(&fixture, CLAUDE_PROJECT);

        assert_eq!(code, 1, "{stdout}");
        assert!(stdout.contains("is a symlink instead of the expected regular file"));
    }
}

#[test]
fn missing_cyclic_and_dangling_includes_fail_closed() {
    let fixture = configured_fixture();
    fs::remove_file(context(&fixture)).unwrap();
    assert_error(
        &fixture,
        "include harness/prompts/fragments/context.md is missing",
    );

    let fixture = configured_fixture();
    fixture.write_repository(
        "harness/prompts/fragments/nested/details.md",
        "@../context.md\nDetails ${ticket}\n",
    );
    assert_error(&fixture, "include cycle reaches");

    let fixture = configured_fixture();
    fs::remove_file(context(&fixture)).unwrap();
    symlink("missing.md", context(&fixture)).unwrap();
    assert_error(
        &fixture,
        "include harness/prompts/fragments/context.md is a dangling symlink",
    );
}

#[test]
fn variables_referenced_in_sources_or_nested_includes_must_be_declared() {
    let fixture = configured_fixture();
    fixture.write_repository(
        "harness/prompts/deploy.md",
        &format!("{SOURCE}Source $undeclared_source\n"),
    );
    assert_error(
        &fixture,
        "variables undeclared_source are referenced but not declared",
    );

    let fixture = configured_fixture();
    fixture.write_repository(
        "harness/prompts/fragments/nested/details.md",
        "Details ${ticket} for $undeclared_include\n",
    );
    assert_error(
        &fixture,
        "variables undeclared_include are referenced but not declared",
    );
}

#[test]
fn malformed_braced_variables_fail_closed() {
    let fixture = configured_fixture();
    fixture.write_repository(
        "harness/prompts/deploy.md",
        &format!("{SOURCE}Invalid ${{undeclared:-fallback}}\n"),
    );

    assert_error(&fixture, "invalid variable reference");
}

#[test]
fn resolved_source_and_destination_aliases_are_topological_errors() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        project_prompt("one", "claude", "file"),
        project_prompt("two", "claude", "file")
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts));
    fixture.write_repository("harness/prompts/shared.md", "same\n");
    for id in ["one", "two"] {
        symlink(
            "shared.md",
            fixture
                .repository()
                .join(format!("harness/prompts/{id}.md")),
        )
        .unwrap();
        fixture.write_repository(format!(".claude/commands/{id}.md"), "same\n");
    }
    assert_error(&fixture, "source two aliases managed source one");

    let fixture = Fixture::new();
    let prompts = prompts
        .replace("commands/one.md", "commands/one/prompt.md")
        .replace("commands/two.md", "commands/two/prompt.md");
    fixture.write_home(".arnes.yaml", &manifest(&prompts));
    for id in ["one", "two"] {
        fixture.write_repository(format!("harness/prompts/{id}.md"), "same\n");
    }
    fixture.write_repository(".claude/commands/shared/prompt.md", "same\n");
    symlink("shared", fixture.repository().join(".claude/commands/one")).unwrap();
    symlink("shared", fixture.repository().join(".claude/commands/two")).unwrap();
    assert_error(
        &fixture,
        "destination .claude/commands/two/prompt.md aliases managed",
    );
}

#[test]
fn resolved_include_aliases_are_topological_errors() {
    let fixture = Fixture::new();
    let prompt = "  - id: deploy\n    source: { root: repository, path: harness/prompts/deploy.md }\n    includes: [fragments/one.md, fragments/two.md]\n    variables: []\n    projections:\n      - agent: claude\n        scope: project\n        representation: file\n        destination: { root: repository, path: .claude/commands/deploy.md }\n";
    fixture.write_home(".arnes.yaml", &manifest(prompt));
    let source = "@fragments/one.md\n@fragments/two.md\nDeploy\n";
    fixture.write_repository("harness/prompts/deploy.md", source);
    fixture.write_repository("harness/prompts/fragments/shared.md", "Shared\n");
    for id in ["one", "two"] {
        symlink(
            "shared.md",
            fixture
                .repository()
                .join(format!("harness/prompts/fragments/{id}.md")),
        )
        .unwrap();
    }
    fixture.write_repository(".claude/commands/deploy.md", source);

    assert_error(&fixture, "aliases another include");
}

fn assert_error(fixture: &Fixture, expected: &str) {
    let (code, stdout, stderr) = run(fixture, CLAUDE_PROJECT);
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    assert!(stderr.is_empty());
}

fn assert_drift(fixture: &Fixture, expected: &str) {
    let (code, stdout, stderr) = run(fixture, CLAUDE_PROJECT);
    assert_eq!(code, 1, "{stdout}");
    assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    assert!(stderr.is_empty());
}

fn source(fixture: &Fixture) -> std::path::PathBuf {
    fixture.repository().join("harness/prompts/deploy.md")
}

fn context(fixture: &Fixture) -> std::path::PathBuf {
    fixture
        .repository()
        .join("harness/prompts/fragments/context.md")
}

fn destination(fixture: &Fixture) -> std::path::PathBuf {
    fixture.repository().join(".claude/commands/deploy.md")
}
