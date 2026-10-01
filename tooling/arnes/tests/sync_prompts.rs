#![cfg(test)]
pub mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, symlink};
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;
const SOURCE: &str = "---\ndescription: Review changes\n---\n@context.md\nReview $TARGET\n";
const MANIFEST: &str = "version: 1
agents:
  - { id: claude, scopes: [user, project] }
  - { id: cursor, scopes: [user, project] }
  - { id: codex, scopes: [user, project] }
resources: []
prompts:
  - id: review
    source: { root: repository, path: harness/prompts/review.md }
    includes: [context.md]
    variables: [TARGET]
    projections:
      - { agent: claude, scope: user, representation: rendered, destination: { root: home, path: .claude/commands/review.md } }
      - { agent: claude, scope: project, representation: file, destination: { root: repository, path: .claude/commands/review.md } }
      - { agent: cursor, scope: project, representation: rendered, destination: { root: repository, path: .cursor/commands/review.md } }
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
    fixture.write_repository("harness/prompts/context.md", "Context\n")?;
    Ok(fixture)
}

#[test]
fn publishes_file_and_rendered_prompts_without_expanding_named_variables() -> TestResult {
    for (agent, scope, directory, expected) in [
        (
            "claude",
            "user",
            ".claude",
            "---\ndescription: Review changes\n---\nReview $TARGET\nContext\n",
        ),
        ("claude", "project", ".claude", SOURCE),
        (
            "cursor",
            "project",
            ".cursor",
            "---\ndescription: Review changes\n---\nReview $TARGET\nContext\n",
        ),
    ] {
        let fixture = fixture()?;
        let output = fixture.command(["sync", "prompts", "--agent", agent, "--scope", scope])?;
        assert!(
            output.status.success(),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let root = if scope == "user" {
            fixture.home()
        } else {
            fixture.repository()
        };
        let path = root.join(directory).join("commands/review.md");
        assert_eq!(fs::read_to_string(&path)?, expected);
        let doctor = fixture.command(["doctor", "prompts", "--agent", agent, "--scope", scope])?;
        assert!(
            doctor.status.success(),
            "{}",
            String::from_utf8_lossy(&doctor.stdout)
        );
        let inode = fs::metadata(&path)?.ino();
        let before = fixture.snapshot()?;
        let replay = fixture.command(["sync", "prompts", "--agent", agent, "--scope", scope])?;
        assert!(replay.status.success());
        assert_eq!(fixture.snapshot()?, before);
        assert_eq!(fs::metadata(path)?.ino(), inode);
        assert_eq!(
            fs::read_to_string(fixture.repository().join("harness/prompts/review.md"))?,
            SOURCE
        );
    }
    Ok(())
}

#[test]
fn updates_proven_rendered_content_and_preserves_unowned_divergence() -> TestResult {
    let fixture = fixture()?;
    let args = ["sync", "prompts", "--agent", "claude", "--scope", "user"];
    assert!(fixture.command(args)?.status.success());
    fixture.write_repository("harness/prompts/context.md", "Updated context\n")?;
    assert!(fixture.command(args)?.status.success());
    assert!(
        fs::read_to_string(fixture.home().join(".claude/commands/review.md"))?
            .ends_with("Updated context\n")
    );
    fixture.write_home(".claude/commands/review.md", "Local prompt\n")?;
    let before = fixture.snapshot()?;
    assert!(!fixture.command(args)?.status.success());
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn invalid_sources_includes_variables_and_destination_aliases_are_refused_before_writing()
-> TestResult {
    for kind in [
        "missing",
        "cycle",
        "variable",
        "outside",
        "symlink",
        "hardlink",
        "description",
    ] {
        let fixture = fixture()?;
        match kind {
            "missing" => fs::remove_file(fixture.repository().join("harness/prompts/context.md"))?,
            "cycle" => fixture.write_repository("harness/prompts/context.md", "@review.md\n")?,
            "variable" => fixture.write_repository("harness/prompts/context.md", "$MISSING\n")?,
            "outside" => {
                fixture.write_repository("harness/prompts/review.md", "@../../../outside.md\n")?;
            }
            "symlink" => symlink(fixture.repository(), fixture.home().join(".claude"))?,
            "hardlink" => {
                fixture.write_home(".claude/commands/neighbor", "keep\n")?;
                fs::hard_link(
                    fixture.repository().join("harness/prompts/review.md"),
                    fixture.home().join(".claude/commands/review.md"),
                )?;
            }
            "description" => fixture.write_home(
                ".arnes.yaml",
                &MANIFEST.replace(
                    "description: Review changes",
                    "description: Different command",
                ),
            )?,
            _ => return Err("invalid fixture".into()),
        }
        let before = fixture.snapshot()?;
        let output =
            fixture.command(["sync", "prompts", "--agent", "claude", "--scope", "user"])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn conforming_regular_prompt_without_receipt_is_never_adopted() -> TestResult {
    let fixture = fixture()?;
    fixture.write_repository(".claude/commands/review.md", SOURCE)?;
    let args = ["sync", "prompts", "--agent", "claude", "--scope", "project"];
    let before = fixture.snapshot()?;
    assert!(fixture.command(args)?.status.success());
    assert_eq!(fixture.snapshot()?, before);
    fixture.write_repository(
        "harness/prompts/review.md",
        &SOURCE.replace("Review $TARGET", "Review selected $TARGET"),
    )?;
    let before = fixture.snapshot()?;
    assert!(!fixture.command(args)?.status.success());
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn unsupported_prompt_representations_and_combinations_do_not_mutate() -> TestResult {
    let fixture = fixture()?;
    for (agent, scope) in [("codex", "user"), ("codex", "project"), ("cursor", "user")] {
        let before = fixture.snapshot()?;
        assert!(
            !fixture
                .command(["sync", "prompts", "--agent", agent, "--scope", scope])?
                .status
                .success()
        );
        assert_eq!(fixture.snapshot()?, before);
    }
    fixture.write_home(
        ".arnes.yaml",
        &MANIFEST.replace("representation: rendered", "representation: symlink"),
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["sync", "prompts", "--agent", "cursor", "--scope", "project"])?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Unsupported"));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn missing_malformed_or_wrong_identity_receipts_never_authorize_stale_updates() -> TestResult {
    for kind in ["missing", "malformed", "replacement"] {
        let fixture = fixture()?;
        let args = ["sync", "prompts", "--agent", "claude", "--scope", "user"];
        assert!(fixture.command(args)?.status.success());
        let artifact = fixture.home().join(".claude/commands/review.md");
        let receipt = fixture
            .home()
            .join(".claude/commands/.review.md.arnes.json");
        match kind {
            "missing" => fs::remove_file(&receipt)?,
            "malformed" => fs::write(&receipt, "private-data-not-for-diagnostics")?,
            "replacement" => {
                let replacement = artifact.with_extension("replacement");
                fs::copy(&artifact, &replacement)?;
                fs::rename(replacement, &artifact)?;
            }
            _ => return Err("invalid fixture".into()),
        }
        fixture.write_repository("harness/prompts/context.md", "Updated context\n")?;
        let before = fixture.snapshot()?;
        let output = fixture.command(args)?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
        assert!(
            !String::from_utf8_lossy(&output.stdout).contains("private-data-not-for-diagnostics")
        );
        assert!(
            !String::from_utf8_lossy(&output.stderr).contains("private-data-not-for-diagnostics")
        );
    }
    Ok(())
}

#[test]
fn publication_failure_does_not_leave_a_truncated_prompt() -> TestResult {
    const LARGE_BODY: usize = 2_000_000;
    let fixture = fixture()?;
    let oversized = format!("{SOURCE}{}", "x".repeat(LARGE_BODY));
    fixture.write_repository("harness/prompts/review.md", &oversized)?;
    fixture.write_home(".claude/commands/neighbor.md", "Local neighbor\n")?;
    let directory = fixture.home().join(".claude/commands");
    let output = fixture.command(["sync", "prompts", "--agent", "claude", "--scope", "user"])?;
    assert!(!output.status.success());
    assert!(!directory.join("review.md").exists());
    assert_eq!(
        fs::read_to_string(directory.join("neighbor.md"))?,
        "Local neighbor\n"
    );
    assert_eq!(
        fs::read_to_string(fixture.repository().join("harness/prompts/review.md"))?,
        oversized
    );
    fixture.write_repository("harness/prompts/review.md", SOURCE)?;
    assert!(
        fixture
            .command(["sync", "prompts", "--agent", "claude", "--scope", "user"])?
            .status
            .success()
    );
    let doctor = fixture.command(["doctor", "prompts", "--agent", "claude", "--scope", "user"])?;
    assert!(doctor.status.success());
    Ok(())
}

#[test]
fn publication_lock_names_cannot_alias_another_canonical_prompt_source() -> TestResult {
    for name in [".review.md.lock", "..review.md.arnes.json.lock"] {
        let fixture = fixture()?;
        let source = format!(".claude/commands/{name}");
        fixture.write_repository(&source, "Other canonical prompt\n")?;
        fixture.write_home(
            ".arnes.yaml",
            &MANIFEST.replace(
                "\ncommands:",
                &format!(
                    "\n  - id: reserved-source
    source: {{ root: repository, path: {source} }}
    includes: []
    variables: []
    projections: []
commands:"
                ),
            ),
        )?;
        let before = fixture.snapshot()?;
        let output =
            fixture.command(["sync", "prompts", "--agent", "claude", "--scope", "project"])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

fn symlink_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = fixture()?;
    fixture.write_home(
        ".arnes.yaml",
        &MANIFEST
            .replace("representation: rendered", "representation: symlink")
            .replace("representation: file", "representation: symlink"),
    )?;
    Ok(fixture)
}

#[test]
fn publishes_claude_links_and_replays_without_receipts_or_rewriting() -> TestResult {
    for scope in ["user", "project"] {
        let fixture = symlink_fixture()?;
        let root = if scope == "user" {
            fixture.home()
        } else {
            fixture.repository()
        };
        let path = root.join(".claude/commands/review.md");
        let args = ["sync", "prompts", "--agent", "claude", "--scope", scope];
        let output = fixture.command(args)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert_eq!(
            fs::canonicalize(&path)?,
            fs::canonicalize(fixture.repository().join("harness/prompts/review.md"))?
        );
        assert_eq!(fs::read_to_string(&path)?, SOURCE);
        let inode = fs::symlink_metadata(&path)?.ino();
        let before = fixture.snapshot()?;
        assert!(fixture.command(args)?.status.success());
        assert_eq!(fixture.snapshot()?, before);
        assert_eq!(fs::symlink_metadata(&path)?.ino(), inode);
        assert!(!root.join(".claude/commands/.review.md.arnes.json").exists());
        fixture.write_repository(
            "harness/prompts/review.md",
            &SOURCE.replace("Review $TARGET", "Updated $TARGET"),
        )?;
        let before = fixture.snapshot()?;
        assert!(fixture.command(args)?.status.success());
        assert_eq!(fixture.snapshot()?, before);
        for resource in ["prompts", "commands"] {
            assert!(
                fixture
                    .command(["doctor", resource, "--agent", "claude", "--scope", scope])?
                    .status
                    .success()
            );
        }
    }
    Ok(())
}

#[test]
fn invalid_link_inputs_and_foreign_destinations_are_preserved() -> TestResult {
    for scope in ["user", "project"] {
        for kind in [
            "regular",
            "directory",
            "wrong",
            "dangling",
            "missing-source",
            "missing-include",
            "variable",
            "cycle",
            "source-outside",
            "parent-link",
            "hardlink",
            "description",
        ] {
            let fixture = symlink_fixture()?;
            let root = if scope == "user" {
                fixture.home()
            } else {
                fixture.repository()
            };
            let source = fixture.repository().join("harness/prompts/review.md");
            let path = root.join(".claude/commands/review.md");
            fs::create_dir_all(path.parent().ok_or("missing parent")?)?;
            match kind {
                "regular" => fs::write(&path, SOURCE)?,
                "directory" => fs::create_dir(&path)?,
                "wrong" => symlink(
                    fixture.repository().join("harness/prompts/context.md"),
                    &path,
                )?,
                "dangling" => symlink("absent.md", &path)?,
                "missing-source" => fs::remove_file(&source)?,
                "missing-include" => {
                    fs::remove_file(fixture.repository().join("harness/prompts/context.md"))?;
                }
                "variable" => fs::write(&source, "$UNKNOWN\n")?,
                "cycle" => {
                    fixture.write_repository("harness/prompts/context.md", "@review.md\n")?;
                }
                "source-outside" => {
                    fixture.write_home("outside.md", SOURCE)?;
                    fs::remove_file(&source)?;
                    symlink(fixture.home().join("outside.md"), &source)?;
                }
                "parent-link" => {
                    fs::remove_dir_all(root.join(".claude"))?;
                    fs::create_dir(root.join("redirect"))?;
                    symlink(root.join("redirect"), root.join(".claude"))?;
                }
                "hardlink" => fs::hard_link(&source, &path)?,
                "description" => fixture.write_home(
                    ".arnes.yaml",
                    &MANIFEST
                        .replace("representation: rendered", "representation: symlink")
                        .replace("representation: file", "representation: symlink")
                        .replace("description: Review changes", "description: Different"),
                )?,
                _ => return Err("unknown fixture".into()),
            }
            let before = fixture.snapshot()?;
            let output =
                fixture.command(["sync", "prompts", "--agent", "claude", "--scope", scope])?;
            assert!(!output.status.success(), "{scope}/{kind}");
            assert_eq!(fixture.snapshot()?, before, "{scope}/{kind}");
        }
    }
    Ok(())
}

#[test]
fn mixed_markdown_selection_is_prepared_before_any_publication() -> TestResult {
    for (first, second) in [("rendered", "symlink"), ("symlink", "rendered")] {
        let fixture = fixture()?;
        let second_prompt = format!(
            "\n  - id: linked\n    source: {{ root: repository, path: harness/prompts/linked.md }}\n    includes: []\n    variables: []\n    projections:\n      - {{ agent: claude, scope: user, representation: {second}, destination: {{ root: home, path: .claude/commands/linked.md }} }}\ncommands:"
        );
        fixture.write_home(
            ".arnes.yaml",
            &MANIFEST
                .replace(
                    "agent: claude, scope: user, representation: rendered",
                    &format!("agent: claude, scope: user, representation: {first}"),
                )
                .replace("\ncommands:", &second_prompt),
        )?;
        fixture.write_repository("harness/prompts/linked.md", "Linked\n")?;
        fixture.write_home(".claude/commands/linked.md", "Local command\n")?;
        let before = fixture.snapshot()?;
        let output =
            fixture.command(["sync", "prompts", "--agent", "claude", "--scope", "user"])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
        assert!(!fixture.home().join(".claude/commands/review.md").exists());
    }
    Ok(())
}

#[test]
fn link_publication_cannot_alias_another_declared_source() -> TestResult {
    for source in [".claude/commands/review.md", ".claude/commands"] {
        let fixture = symlink_fixture()?;
        let manifest = MANIFEST.replace("representation: rendered", "representation: symlink").replace("representation: file", "representation: symlink").replace("\ncommands:", &format!("\n  - id: reserved\n    source: {{ root: repository, path: {source} }}\n    includes: []\n    variables: []\n    projections: []\ncommands:"));
        fixture.write_home(".arnes.yaml", &manifest)?;
        let before = fixture.snapshot()?;
        let output =
            fixture.command(["sync", "prompts", "--agent", "claude", "--scope", "project"])?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}
