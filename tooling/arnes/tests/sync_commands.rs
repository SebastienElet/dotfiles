#![cfg(test)]
pub mod support;

use std::fs;
use std::os::unix::fs::MetadataExt;
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
const SOURCE: &str = "---\ndescription: Review changes\n---\nReview $TARGET\n";
const MANIFEST: &str = "version: 1
agents:
  - { id: claude, scopes: [user, project] }
  - { id: cursor, scopes: [user, project] }
  - { id: codex, scopes: [user, project] }
resources: []
prompts:
  - id: review
    source: { root: repository, path: harness/prompts/review.md }
    includes: []
    variables: [TARGET]
    projections:
      - { agent: claude, scope: user, representation: rendered, destination: { root: home, path: .claude/commands/review.md } }
      - { agent: claude, scope: project, representation: file, destination: { root: repository, path: .claude/commands/review.md } }
commands:
  - name: review
    description: Review changes
    prompt: review
    bindings:
      - { agent: claude, scope: user }
      - { agent: claude, scope: project }
";

fn fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", MANIFEST)?;
    fixture.write_repository("harness/prompts/review.md", SOURCE)?;
    Ok(fixture)
}

#[test]
fn commands_and_prompts_share_one_artifact_in_either_order() -> TestResult {
    for scope in ["user", "project"] {
        for resources in [["commands", "prompts"], ["prompts", "commands"]] {
            let fixture = fixture()?;
            let root = if scope == "user" {
                fixture.home()
            } else {
                fixture.repository()
            };
            let path = root.join(".claude/commands/review.md");
            let [first, second] = resources;
            assert!(
                fixture
                    .command(["sync", first, "--agent", "claude", "--scope", scope])?
                    .status
                    .success()
            );
            let inode = fs::metadata(&path)?.ino();
            let before = fixture.snapshot()?;
            assert!(
                fixture
                    .command(["sync", second, "--agent", "claude", "--scope", scope])?
                    .status
                    .success()
            );
            assert_eq!(fs::metadata(&path)?.ino(), inode);
            assert_eq!(fixture.snapshot()?, before);
            assert_eq!(fs::read_to_string(&path)?, SOURCE);
            for resource in resources {
                let doctor =
                    fixture.command(["doctor", resource, "--agent", "claude", "--scope", scope])?;
                assert!(
                    doctor.status.success(),
                    "{}",
                    String::from_utf8_lossy(&doctor.stdout)
                );
            }
        }
    }
    Ok(())
}

#[test]
fn refuses_incompatible_names_descriptions_and_unowned_local_commands() -> TestResult {
    for kind in ["name", "description", "missing-prompt", "local"] {
        let fixture = fixture()?;
        match kind {
            "name" => fixture.write_home(
                ".arnes.yaml",
                &MANIFEST.replace("name: review", "name: other"),
            )?,
            "description" => fixture.write_home(
                ".arnes.yaml",
                &MANIFEST.replace("description: Review changes", "description: Other command"),
            )?,
            "missing-prompt" => fixture.write_home(
                ".arnes.yaml",
                &MANIFEST.replace("prompt: review", "prompt: missing"),
            )?,
            "local" => fixture.write_home(".claude/commands/review.md", "Local command\n")?,
            _ => return Err("invalid fixture".into()),
        }
        let before = fixture.snapshot()?;
        assert!(
            !fixture
                .command(["sync", "commands", "--agent", "claude", "--scope", "user"])?
                .status
                .success()
        );
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn command_content_updates_use_the_same_prompt_ownership_receipt() -> TestResult {
    let fixture = fixture()?;
    assert!(
        fixture
            .command(["sync", "prompts", "--agent", "claude", "--scope", "user"])?
            .status
            .success()
    );
    fixture.write_repository(
        "harness/prompts/review.md",
        &SOURCE.replace("Review $TARGET", "Updated $TARGET"),
    )?;
    assert!(
        fixture
            .command(["sync", "commands", "--agent", "claude", "--scope", "user"])?
            .status
            .success()
    );
    let before = fixture.snapshot()?;
    assert!(
        fixture
            .command(["sync", "prompts", "--agent", "claude", "--scope", "user"])?
            .status
            .success()
    );
    assert_eq!(fixture.snapshot()?, before);
    assert!(
        fs::read_to_string(fixture.home().join(".claude/commands/review.md"))?
            .contains("Updated $TARGET")
    );
    Ok(())
}

#[test]
fn unsupported_command_combinations_never_mutate() -> TestResult {
    let fixture = fixture()?;
    for agent in ["cursor", "codex"] {
        for scope in ["user", "project"] {
            let before = fixture.snapshot()?;
            assert!(
                !fixture
                    .command(["sync", "commands", "--agent", agent, "--scope", scope])?
                    .status
                    .success()
            );
            assert_eq!(fixture.snapshot()?, before);
        }
    }
    Ok(())
}
