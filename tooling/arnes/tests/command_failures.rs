#[path = "support/commands.rs"]
pub mod command_support;
#[path = "support/configured_commands.rs"]
mod configured_command_support;
pub mod support;

use command_support::{CONTENTS, command, manifest, output_tuple, prompt, run};
use configured_command_support::configured_fixture;
use std::fs;
use std::os::unix::fs::{PermissionsExt, symlink};
use support::Fixture;

const CLAUDE_PROJECT: &[&str] = &[
    "doctor", "commands", "--agent", "claude", "--scope", "project", "--format", "json",
];

fn assert_state(fixture: &Fixture, code: i32, expected: &str) {
    let (actual, stdout, stderr) = run(fixture, CLAUDE_PROJECT);
    assert_eq!(actual, code, "{stdout}");
    assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    assert!(stderr.is_empty());
}

#[test]
fn missing_wrong_type_stale_and_symlink_destinations_are_drift() {
    let fixture = configured_fixture();
    fs::remove_file(fixture.repository().join(".claude/commands/deploy.md")).unwrap();
    assert_state(&fixture, 1, "is missing");

    let fixture = configured_fixture();
    let destination = fixture.repository().join(".claude/commands/deploy.md");
    fs::remove_file(&destination).unwrap();
    fs::create_dir(&destination).unwrap();
    assert_state(&fixture, 1, "is not a regular file");

    let fixture = configured_fixture();
    fixture.write_repository(".claude/commands/deploy.md", "stale\n");
    assert_state(&fixture, 1, "is stale");

    let fixture = configured_fixture();
    let destination = fixture.repository().join(".claude/commands/deploy.md");
    fs::remove_file(&destination).unwrap();
    symlink("../../harness/prompts/deploy.md", &destination).unwrap();
    assert_state(&fixture, 1, "is a symlink");
}

#[test]
fn unreadable_source_and_destination_are_errors() {
    for relative in ["harness/prompts/deploy.md", ".claude/commands/deploy.md"] {
        let fixture = configured_fixture();
        let before = fixture.snapshot();
        let path = fixture.repository().join(relative);
        let permissions = fs::metadata(&path).unwrap().permissions();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
        let output = fixture.command(CLAUDE_PROJECT);
        fs::set_permissions(&path, permissions).unwrap();
        assert_eq!(fixture.snapshot(), before);
        let (code, stdout, stderr) = output_tuple(output);

        assert_eq!(code, 2, "{stdout}");
        assert!(stdout.contains("could not be read"));
        assert!(stderr.is_empty());
    }
}

#[test]
fn projection_contract_failures_are_explicit() {
    let fixture = Fixture::new();
    let no_projection = prompt(
        "deploy",
        "cursor",
        "project",
        "file",
        ".cursor/commands/deploy.md",
    );
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: project }\n",
    );
    fixture.write_home(".arnes.yaml", &manifest(&no_projection, &commands));
    assert_state(&fixture, 2, "has no projection for this binding");

    let fixture = Fixture::new();
    let incompatible = prompt(
        "deploy",
        "claude",
        "project",
        "file",
        ".claude/commands/not-deploy.md",
    );
    fixture.write_home(".arnes.yaml", &manifest(&incompatible, &commands));
    assert_state(&fixture, 2, "does not match .claude/commands/deploy.md");

    let fixture = Fixture::new();
    let symlinked = prompt(
        "deploy",
        "claude",
        "project",
        "symlink",
        ".claude/commands/deploy.md",
    );
    fixture.write_home(".arnes.yaml", &manifest(&symlinked, &commands));
    assert_state(
        &fixture,
        0,
        "symlink projections have no stable agent contract",
    );
}

#[test]
fn source_include_and_variable_failures_are_transposed() {
    let fixture = configured_fixture();
    fs::remove_file(fixture.repository().join("harness/prompts/deploy.md")).unwrap();
    assert_state(&fixture, 2, "source harness/prompts/deploy.md is missing");

    let fixture = Fixture::new();
    let prompts = "  - id: deploy
    source: { root: repository, path: harness/prompts/deploy.md }
    includes: [missing.md]
    variables: []
    projections:
      - agent: claude
        scope: project
        representation: file
        destination: { root: repository, path: .claude/commands/deploy.md }
";
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: project }\n",
    );
    fixture.write_home(".arnes.yaml", &manifest(prompts, &commands));
    fixture.write_repository("harness/prompts/deploy.md", "@missing.md\nDeploy\n");
    fixture.write_repository(".claude/commands/deploy.md", "@missing.md\nDeploy\n");
    assert_state(&fixture, 2, "include harness/prompts/missing.md is missing");

    let fixture = configured_fixture();
    let contents = format!("{CONTENTS}Deploy $undeclared\n");
    fixture.write_repository("harness/prompts/deploy.md", &contents);
    fixture.write_repository(".claude/commands/deploy.md", &contents);
    assert_state(
        &fixture,
        2,
        "variables undeclared are referenced but not declared",
    );
}

#[test]
fn frontmatter_and_description_mismatches_are_drift() {
    for (contents, expected) in [
        ("Deploy now\n", "frontmatter missing or malformed"),
        (
            "---\ndescription: Deploy safely\nDeploy now\n",
            "frontmatter missing or malformed",
        ),
        (
            "---\ndescription: [\n---\nDeploy now\n",
            "frontmatter missing or malformed",
        ),
        (
            "---\nother: value\n---\nDeploy now\n",
            "description is missing",
        ),
        (
            "---\ndescription: 42\n---\nDeploy now\n",
            "description must be a string",
        ),
        (
            "---\ndescription: Deploy unsafely\n---\nDeploy now\n",
            "description differs from manifest",
        ),
    ] {
        let fixture = configured_fixture();
        fixture.write_repository("harness/prompts/deploy.md", contents);
        fixture.write_repository(".claude/commands/deploy.md", contents);
        assert_state(&fixture, 1, expected);
    }
}

#[test]
fn unrelated_frontmatter_keys_are_ignored() {
    let fixture = configured_fixture();
    let contents = "---\ndescription: Deploy safely\nallowed-tools: Bash\n---\nDeploy now\n";
    fixture.write_repository("harness/prompts/deploy.md", contents);
    fixture.write_repository(".claude/commands/deploy.md", contents);
    assert_state(&fixture, 0, "binding is current");
}

#[test]
fn the_highest_state_controls_the_exit_code() {
    let fixture = Fixture::new();
    let prompts = "  - id: cursor-command
    source: { root: repository, path: harness/prompts/cursor-command.md }
    includes: []
    variables: []
    projections: []
  - id: drifting
    source: { root: repository, path: harness/prompts/drifting.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: file
        destination: { root: home, path: .claude/commands/drifting.md }
  - id: broken
    source: { root: repository, path: harness/prompts/broken.md }
    includes: []
    variables: []
    projections:
      - agent: claude
        scope: user
        representation: file
        destination: { root: home, path: .claude/commands/broken.md }
";
    let commands = "  - name: cursor-command
    description: Deploy safely
    prompt: cursor-command
    bindings: [{ agent: cursor, scope: user }]
  - name: drifting
    description: Deploy safely
    prompt: drifting
    bindings: [{ agent: claude, scope: user }]
  - name: broken
    description: Deploy safely
    prompt: broken
    bindings: [{ agent: claude, scope: user }]
";
    fixture.write_home(".arnes.yaml", &manifest(prompts, commands));
    fixture.write_repository("harness/prompts/drifting.md", CONTENTS);
    let (code, stdout, stderr) = run(&fixture, &["doctor", "commands"]);

    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains("unsupported"));
    assert!(stdout.contains("drift"));
    assert!(stdout.contains("error"));
    assert!(stderr.is_empty());
}
