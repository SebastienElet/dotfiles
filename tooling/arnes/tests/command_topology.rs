#[path = "support/commands.rs"]
mod command_support;
mod support;

use command_support::{CONTENTS, command, manifest, output_tuple, prompt, run};
use std::fs;
use std::os::unix::fs::symlink;
use std::process::Command;
use support::Fixture;

const CLAUDE_USER: &[&str] = &[
    "doctor", "commands", "--agent", "claude", "--scope", "user", "--format", "json",
];

#[test]
fn selected_command_destinations_cannot_alias_each_other() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt("one", "claude", "user", "file", ".claude/commands/one.md"),
        prompt("two", "claude", "user", "file", ".claude/commands/two.md")
    );
    let commands = format!(
        "{}{}",
        command("one", "one", "      - { agent: claude, scope: user }\n"),
        command("two", "two", "      - { agent: claude, scope: user }\n")
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    for id in ["one", "two"] {
        let source = fixture
            .repository()
            .join(format!("harness/prompts/{id}.md"));
        fs::create_dir_all(source.parent().unwrap()).unwrap();
        fs::write(source, CONTENTS).unwrap();
    }
    fixture.write_home(".claude/commands/shared.md", CONTENTS);
    symlink("shared.md", fixture.home().join(".claude/commands/one.md")).unwrap();
    symlink("shared.md", fixture.home().join(".claude/commands/two.md")).unwrap();

    assert_collision(&fixture, "aliases managed destination");
}

#[test]
fn command_destinations_cannot_alias_managed_resources() {
    let fixture = Fixture::new();
    let prompts = prompt(
        "deploy",
        "claude",
        "user",
        "file",
        ".claude/commands/deploy.md",
    );
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: user }\n",
    );
    let configured = manifest(&prompts, &commands).replace(
        "resources: []",
        "resources:\n  - id: managed-resource\n    kind: instructions\n    agent: claude\n    scope: user\n    source: { root: repository, path: harness/AGENTS.md }\n    destination: { root: home, path: .claude/commands/resource.md }",
    );
    fixture.write_home(".arnes.yaml", &configured);
    fixture.write_repository("harness/prompts/deploy.md", CONTENTS);
    fixture.write_home(".claude/commands/shared.md", CONTENTS);
    symlink(
        "shared.md",
        fixture.home().join(".claude/commands/deploy.md"),
    )
    .unwrap();
    symlink(
        "shared.md",
        fixture.home().join(".claude/commands/resource.md"),
    )
    .unwrap();

    assert_collision(&fixture, "aliases managed destination resource");
}

#[test]
fn command_destinations_cannot_alias_unreferenced_prompt_projections() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt(
            "deploy",
            "claude",
            "user",
            "file",
            ".claude/commands/deploy.md"
        ),
        prompt(
            "other",
            "claude",
            "user",
            "file",
            ".claude/commands/alias/deploy.md"
        )
    );
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: user }\n",
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    fixture.write_repository("harness/prompts/deploy.md", CONTENTS);
    fixture.write_repository("harness/prompts/other.md", CONTENTS);
    fixture.write_home(".claude/commands/deploy.md", CONTENTS);
    symlink(".", fixture.home().join(".claude/commands/alias")).unwrap();

    assert_collision(&fixture, "aliases managed destination");
}

#[test]
fn hardlinked_command_destinations_cannot_alias_each_other() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt("one", "claude", "user", "file", ".claude/commands/one.md"),
        prompt("two", "claude", "user", "file", ".claude/commands/two.md")
    );
    let commands = format!(
        "{}{}",
        command("one", "one", "      - { agent: claude, scope: user }\n"),
        command("two", "two", "      - { agent: claude, scope: user }\n")
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts, &commands));
    fixture.write_repository("harness/prompts/one.md", CONTENTS);
    fixture.write_repository("harness/prompts/two.md", CONTENTS);
    fixture.write_home(".claude/commands/one.md", CONTENTS);
    fs::hard_link(
        fixture.home().join(".claude/commands/one.md"),
        fixture.home().join(".claude/commands/two.md"),
    )
    .unwrap();

    assert_collision(&fixture, "aliases managed destination");
}

#[test]
fn shared_roots_cannot_hide_cross_scope_resource_collisions() {
    let fixture = Fixture::new();
    let prompts = prompt(
        "deploy",
        "claude",
        "user",
        "file",
        ".claude/commands/deploy.md",
    );
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: user }\n",
    );
    let configured = manifest(&prompts, &commands).replace(
        "resources: []",
        "resources:\n  - id: managed-resource\n    kind: instructions\n    agent: claude\n    scope: project\n    source: { root: repository, path: harness/AGENTS.md }\n    destination: { root: repository, path: .claude/commands/deploy.md }",
    );
    fixture.write_repository(".arnes.yaml", &configured);
    fixture.write_repository("harness/prompts/deploy.md", CONTENTS);
    fixture.write_repository(".claude/commands/deploy.md", CONTENTS);

    assert_shared_root_collision(&fixture, "aliases managed destination resource");
}

#[test]
fn shared_roots_cannot_hide_cross_scope_prompt_collisions() {
    let fixture = Fixture::new();
    let prompts = format!(
        "{}{}",
        prompt(
            "deploy",
            "claude",
            "user",
            "file",
            ".claude/commands/deploy.md"
        ),
        prompt(
            "other",
            "claude",
            "project",
            "file",
            ".claude/commands/deploy.md"
        )
    );
    let commands = command(
        "deploy",
        "deploy",
        "      - { agent: claude, scope: user }\n",
    );
    fixture.write_repository(".arnes.yaml", &manifest(&prompts, &commands));
    fixture.write_repository("harness/prompts/deploy.md", CONTENTS);
    fixture.write_repository("harness/prompts/other.md", CONTENTS);
    fixture.write_repository(".claude/commands/deploy.md", CONTENTS);

    assert_shared_root_collision(&fixture, "aliases managed destination");
}

fn assert_collision(fixture: &Fixture, expected: &str) {
    let (code, stdout, stderr) = run(fixture, CLAUDE_USER);
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    assert!(stderr.is_empty());
}

fn assert_shared_root_collision(fixture: &Fixture, expected: &str) {
    let before = fixture.snapshot();
    let output = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(CLAUDE_USER)
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.repository())
        .output()
        .unwrap();
    assert_eq!(fixture.snapshot(), before);
    let (code, stdout, stderr) = output_tuple(output);
    assert_eq!(code, 2, "{stdout}");
    assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    assert!(stderr.is_empty());
}
