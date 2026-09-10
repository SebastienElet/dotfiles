#![cfg(test)]
#[path = "support/prompts.rs"]
pub mod prompt_support;
pub mod support;
use prompt_support::{configured_fixture, manifest, project_prompt, run};
use serde_json::Value;
use support::Fixture;
#[test]
fn direct_project_files_are_healthy_for_claude_and_cursor()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for agent in ["claude", "cursor"] {
        let fixture = configured_fixture()?;
        let (code, stdout, stderr) = run(
            &fixture,
            &[
                "doctor", "prompts", "--agent", agent, "--scope", "project", "-v",
            ],
        )?;
        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains(&format!("{agent} project prompts")));
        assert!(stdout.contains("healthy     deploy · current"));
        assert!(stderr.is_empty());
    };
    Ok(())
}
#[test]
fn prompt_content_does_not_validate_command_names_or_metadata()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    let prompt = "  - id: not a slash command\n    source: { root: repository, path: harness/prompts/content.md }\n    includes: []\n    variables: []\n    projections:\n      - agent: claude\n        scope: project\n        representation: file\n        destination: { root: repository, path: .claude/commands/not-registered.md }\n";
    fixture.write_home(".arnes.yaml", &manifest(prompt))?;
    fixture.write_repository("harness/prompts/content.md", "plain content\n")?;
    fixture.write_repository(".claude/commands/not-registered.md", "plain content\n")?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "project", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy     not a slash command · current"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn rendered_user_file_resolves_nested_includes_and_declared_variables()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "user", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("healthy     deploy · current"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn unsupported_agent_scope_combinations_do_not_inspect_prompts()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (agent, scope) in [("cursor", "user"), ("codex", "user"), ("codex", "project")]
    {
        let fixture = Fixture::new()?;
        fixture.write_home(
            ".arnes.yaml",
            &manifest(&project_prompt("missing", "claude", "file")),
        )?;
        let (code, stdout, stderr) = run(
            &fixture,
            &["doctor", "prompts", "--agent", agent, "--scope", scope],
        )?;
        assert_eq!(code, 0, "{stdout}");
        assert!(stdout.contains("capability · unsupported"));
        assert!(!stdout.contains("source"));
        assert!(stderr.is_empty());
    };
    Ok(())
}
#[test]
fn supported_combinations_without_managed_projections_are_explicitly_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest(""))?;
    let (code, stdout, stderr) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "project", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("capability · unsupported"));
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn symlink_representation_is_unsupported_without_inspecting_its_source()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        &manifest(&project_prompt("missing", "claude", "symlink")),
    )?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "project", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("symlink projection unsupported"));
    assert!(!stdout.contains("source"));
    Ok(())
}
#[test]
fn filters_prevent_io_for_unselected_prompt_projections()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    let prompts = format!(
        "{}{}",
        project_prompt("selected", "claude", "file"),
        project_prompt("unselected", "cursor", "file")
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts))?;
    fixture.write_repository("harness/prompts/selected.md", "selected\n")?;
    fixture.write_repository(".claude/commands/selected.md", "selected\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "project", "-v",
        ],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(stdout.contains("selected · current"));
    assert!(!stdout.contains("unselected"));
    Ok(())
}
#[test]
fn human_and_json_preserve_manifest_prompt_order()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    let prompts = format!(
        "{}{}",
        project_prompt("zeta", "claude", "file"),
        project_prompt("alpha", "claude", "file")
    );
    fixture.write_home(".arnes.yaml", &manifest(&prompts))?;
    for id in ["zeta", "alpha"] {
        fixture.write_repository(format!("harness/prompts/{id}.md"), id)?;
        fixture.write_repository(format!(".claude/commands/{id}.md"), id)?;
    }
    let (_, human, _) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "project", "-v",
        ],
    )?;
    let (_, json, _) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "project", "--format", "json",
        ],
    )?;
    assert!(
        human
            .find("zeta · current")
            .ok_or("required test value is missing")?
            < human
                .find("alpha · current")
                .ok_or("required test value is missing")?
    );
    let diagnostics: Vec<Value> = serde_json::from_str(&json)?;
    assert!(
        (*(*(diagnostics).first().ok_or("missing fixture index 0")?)
            .get("message")
            .ok_or("missing fixture index message")?)
        .as_str()
        .ok_or("expected JSON string")?
        .contains("zeta")
    );
    assert!(
        (*(*(diagnostics).get(1).ok_or("missing fixture index 1")?)
            .get("message")
            .ok_or("missing fixture index message")?)
        .as_str()
        .ok_or("expected JSON string")?
        .contains("alpha")
    );
    Ok(())
}
#[test]
fn unmanaged_and_plugin_owned_commands_are_ignored_and_preserved()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".claude/commands/unmanaged.md", "$MISSING\n")?;
    fixture.write_home(".claude/commands/opsx/apply.md", "plugin owned\n")?;
    fixture.write_repository(".claude/commands/opsx/project.md", "plugin owned\n")?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "prompts", "--agent", "claude", "--scope", "user"],
    )?;
    assert_eq!(code, 0, "{stdout}");
    assert!(!stdout.contains("unmanaged"));
    assert!(!stdout.contains("opsx"));
    Ok(())
}
#[test]
fn default_doctor_reuses_filtered_prompt_diagnostics()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_home(".claude/commands/deploy.md", "stale\n")?;
    let (_, direct, _) = run(
        &fixture,
        &[
            "doctor", "prompts", "--agent", "claude", "--scope", "user", "--format", "json",
        ],
    )?;
    let (_, aggregate, _) = run(
        &fixture,
        &[
            "doctor", "--agent", "claude", "--scope", "user", "--format", "json",
        ],
    )?;
    let direct: Vec<Value> = serde_json::from_str(&direct)?;
    let aggregate: Vec<Value> = serde_json::from_str(&aggregate)?;
    let aggregate = aggregate
        .into_iter()
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(diagnostic)
                        .get("resource")
                        .ok_or("missing fixture index resource")?
                        == "prompts"
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
