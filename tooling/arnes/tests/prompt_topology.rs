#[path = "support/prompts.rs"]
pub mod prompt_support;
pub mod support;

use prompt_support::{manifest, project_prompt, run};
use std::fs;
use std::os::unix::fs::symlink;
use support::Fixture;

const CLAUDE_PROJECT: &[&str] = &[
    "doctor", "prompts", "--agent", "claude", "--scope", "project", "--format", "json",
];

const CLAUDE_USER: &[&str] = &[
    "doctor", "prompts", "--agent", "claude", "--scope", "user", "--format", "json",
];

#[test]
fn prompt_destinations_cannot_alias_other_managed_resources() {
    let fixture = Fixture::new();
    let prompt = project_prompt("prompt/prompt", "claude", "file")
        .replace("prompts/prompt/prompt.md", "prompts/prompt.md");
    let manifest = manifest(&prompt).replace(
        "resources: []",
        "resources:\n  - id: managed-resource\n    kind: instructions\n    agent: claude\n    scope: project\n    source: { root: repository, path: harness/AGENTS.md }\n    destination: { root: repository, path: .claude/commands/resource/prompt.md }",
    );
    fixture.write_home(".arnes.yaml", &manifest);
    fixture.write_repository("harness/prompts/prompt.md", "same\n");
    fixture.write_repository(".claude/commands/shared/prompt.md", "same\n");
    symlink(
        "shared",
        fixture.repository().join(".claude/commands/prompt"),
    )
    .unwrap();
    symlink(
        "shared",
        fixture.repository().join(".claude/commands/resource"),
    )
    .unwrap();

    assert_collision(&fixture, "aliases managed destination resource");
}

#[test]
fn absent_prompt_destinations_collide_after_parent_resolution() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        project_prompt("one/prompt", "claude", "file")
            .replace("prompts/one/prompt.md", "prompts/one.md"),
        project_prompt("two/prompt", "claude", "file")
            .replace("prompts/two/prompt.md", "prompts/two.md"),
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts));
    for id in ["one", "two"] {
        fixture.write_repository(format!("harness/prompts/{id}.md"), "same\n");
    }
    fs::create_dir_all(fixture.repository().join(".claude/commands/shared")).unwrap();
    symlink("shared", fixture.repository().join(".claude/commands/one")).unwrap();
    symlink("shared", fixture.repository().join(".claude/commands/two")).unwrap();

    assert_collision(&fixture, "aliases managed destination");
}

#[test]
fn resolved_source_destination_aliases_only_allow_direct_project_files() {
    for representation in ["file", "rendered"] {
        let fixture = Fixture::new();
        let prompt = project_prompt("deploy", "claude", representation);
        fixture.write_home(".arnes.yaml", &manifest(&prompt));
        fixture.write_repository("harness/prompts/deploy.md", "same\n");
        fs::create_dir_all(fixture.repository().join(".claude")).unwrap();
        symlink(
            "../harness/prompts",
            fixture.repository().join(".claude/commands"),
        )
        .unwrap();

        let (code, stdout, stderr) = run(&fixture, CLAUDE_PROJECT);

        if representation == "file" {
            assert_eq!(code, 0, "{stdout}");
            assert!(stdout.contains("\"state\": \"healthy\""));
        } else {
            assert_eq!(code, 2, "{stdout}");
            assert!(stdout.contains("aliases managed destination deploy"));
        }
        assert!(stderr.is_empty());
    }
}

#[test]
fn filtered_scopes_ignore_hardlinked_resources_in_distinct_roots() {
    let fixture = Fixture::new();
    let prompt = "  - id: deploy
    source: { root: repository, path: harness/prompts/deploy.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: rendered
        destination: { root: home, path: .claude/commands/deploy.md }
";
    let configured = manifest(prompt).replace(
        "resources: []",
        "resources:\n  - id: managed-resource\n    kind: instructions\n    agent: claude\n    scope: project\n    source: { root: repository, path: harness/AGENTS.md }\n    destination: { root: repository, path: .claude/resource.md }",
    );
    fixture.write_home(".arnes.yaml", &configured);
    fixture.write_repository("harness/prompts/deploy.md", "same\n");
    fixture.write_repository(".claude/resource.md", "same\n");
    fs::create_dir_all(fixture.home().join(".claude/commands")).unwrap();
    fs::hard_link(
        fixture.repository().join(".claude/resource.md"),
        fixture.home().join(".claude/commands/deploy.md"),
    )
    .unwrap();

    let (code, stdout, stderr) = run(&fixture, CLAUDE_USER);
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("\"state\": \"healthy\""));
    assert!(stderr.is_empty());
}

fn assert_collision(fixture: &Fixture, expected: &str) {
    let (code, stdout, stderr) = run(fixture, CLAUDE_PROJECT);
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    assert!(stderr.is_empty());
}
