#![cfg(test)]
#[path = "support/rules.rs"]
pub mod rule_support;
pub mod support;
use rule_support::{configured_fixture, run};
use std::fs;
use std::os::unix::fs::symlink;
#[test]
fn claude_user_rule_symlinks_are_healthy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "rules", "--agent", "claude", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy rules: claude user rule agent-instructions"));
    assert!(stdout.contains("destination ~/.claude/rules/agent-instructions.md is current"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn cursor_user_rule_symlinks_are_healthy() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    fixture . write_home (".arnes.yaml" , "version: 1\nagents:\n  - id: cursor\n    scopes: [user]\nresources:\n  - id: memory-governance-cursor\n    kind: rules\n    agent: cursor\n    scope: user\n    source: { root: repository, path: harness/rules/memory-governance-cursor.mdc }\n    destination: { root: home, path: .cursor/rules/memory-governance-cursor.mdc }\n" ,) ? ;
    fixture.write_repository(
        "harness/rules/memory-governance-cursor.mdc",
        "---\nalwaysApply: true\n---\n",
    )?;
    let destination = fixture
        .home()
        .join(".cursor/rules/memory-governance-cursor.mdc");
    fs::create_dir_all(destination.parent().ok_or("fixture path has no parent")?)?;
    symlink(
        fs::canonicalize(
            fixture
                .repository()
                .join("harness/rules/memory-governance-cursor.mdc"),
        )?,
        destination,
    )?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "rules", "--agent", "cursor", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy rules: cursor user rule memory-governance-cursor"));
    assert!(stdout.contains("destination ~/.cursor/rules/memory-governance-cursor.mdc is current"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn filters_isolate_rules_and_report_unsupported_capabilities()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "codex", "--scope", "user"],
    )?;
    assert_eq!(code, 0);
    assert!(stdout.contains("codex user rule projection is not declared or supported"));
    assert!(!stdout.contains("agent-instructions"));
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "claude", "--scope", "project"],
    )?;
    assert_eq!(code, 0);
    assert!(stdout.contains("claude project rule projection is not declared or supported"));
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "codex", "--scope", "project"],
    )?;
    assert_eq!(code, 0);
    assert!(stdout.contains("codex project rule projection is not declared or supported"));
    Ok(())
}
#[test]
fn undeclared_filters_are_explicitly_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = support::Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: claude\n    scopes: [user]\nresources: []\n",
    )?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "rules", "--agent", "codex"])?;
    assert_eq!(code, 0);
    assert!(stdout.contains("unsupported rules: codex user rule projection"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn rule_diagnostics_use_the_rules_resource() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "rules", "--agent", "claude", "--format", "json"],
    )?;
    assert_eq!(code, 0);
    let diagnostics: serde_json::Value = serde_json::from_str(&stdout)?;
    assert_eq!(
        *(*(diagnostics).get(0).ok_or("missing fixture index 0")?)
            .get("resource")
            .ok_or("missing fixture index resource")?,
        "rules"
    );
    assert_eq!(
        *(*(diagnostics).get(0).ok_or("missing fixture index 0")?)
            .get("state")
            .ok_or("missing fixture index state")?,
        "healthy"
    );
    Ok(())
}
#[test]
fn rules_doctor_is_read_only() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let before = fixture.snapshot()?;
    let (code, _, _) = run(&fixture, &["doctor", "rules"])?;
    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
#[test]
fn default_doctor_reuses_filtered_rule_diagnostics()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (_, direct, _) = run(
        &fixture,
        &[
            "doctor", "rules", "--agent", "claude", "--scope", "user", "--format", "json",
        ],
    )?;
    let (_, aggregate, _) = run(
        &fixture,
        &[
            "doctor", "--agent", "claude", "--scope", "user", "--format", "json",
        ],
    )?;
    let direct: Vec<serde_json::Value> = serde_json::from_str(&direct)?;
    let aggregate: Vec<serde_json::Value> = serde_json::from_str(&aggregate)?;
    let aggregate = aggregate
        .into_iter()
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(diagnostic)
                        .get("resource")
                        .ok_or("missing fixture index resource")?
                        == "rules"
                };
                Ok((include, diagnostic))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .collect::<Vec<_>>();
    assert_eq!(aggregate, direct);
    Ok(())
}
