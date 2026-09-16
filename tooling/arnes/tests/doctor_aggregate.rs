#![cfg(test)]
#[path = "support/doctor_aggregate.rs"]
mod aggregate_support;
mod support;
use aggregate_support::{ORDER, configured_fixture, set_mode};
use serde_json::Value;
use std::fs;
fn json(
    output: &std::process::Output,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&output.stdout)?)
}
#[test]
fn healthy_aggregate_is_deterministic_ordered_and_read_only()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let before = fixture.snapshot()?;
    let human = fixture.command(["doctor", "-v"])?;
    assert_eq!(fixture.snapshot()?, before);
    let repeated_human = fixture.command(["doctor", "-v"])?;
    assert_eq!(fixture.snapshot()?, before);
    let structured = fixture.command(["doctor", "--format", "json"])?;
    assert_eq!(fixture.snapshot()?, before);
    let repeated_structured = fixture.command(["doctor", "--format", "json"])?;
    assert_eq!(
        human.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&human.stdout)
    );
    assert_eq!(human.stdout, repeated_human.stdout);
    assert_eq!(structured.status.code(), Some(0));
    assert_eq!(structured.stdout, repeated_structured.stdout);
    assert_eq!(fixture.snapshot()?, before);
    let human = String::from_utf8(human.stdout)?;
    let positions = [
        "Manifest\n",
        "\n\nConfig · user scope\n",
        "\n\nInstructions · user scope\n",
        "\n\nSkills · user scope · 1 agent\n",
        "\n\nPrompts · user scope\n",
        "\n\nCommands · user scope\n",
        "\n\nRules · user scope\n",
        "\n\nHooks · user scope\n",
        "\n\nMCP\n",
        "\n\nStatusline\n",
    ]
    .map(
        |heading| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            Ok({
                human
                    .find(heading)
                    .ok_or_else(|| format!("missing {heading:?}: {human}"))?
            })
        },
    )
    .into_iter()
    .collect::<Result<Vec<_>, _>>()?;
    assert!(
        positions
            .windows(2)
            .map(
                |pair| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok({
                        *(pair).first().ok_or("missing fixture index 0")?
                            < *(pair).get(1).ok_or("missing fixture index 1")?
                    })
                }
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .all(std::convert::identity)
    );
    let diagnostics = json(&structured)?;
    for resource in ORDER {
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic.get("resource").and_then(Value::as_str) == Some(resource)
                    && diagnostic.get("state").and_then(Value::as_str) == Some("healthy")
            }),
            "missing healthy {resource} diagnostic"
        );
    }
    let groups = diagnostics.iter().try_fold(
        Vec::new(),
        |mut groups, diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            Ok({
                let resource = (*(diagnostic)
                    .get("resource")
                    .ok_or("missing fixture index resource")?)
                .as_str()
                .ok_or("expected JSON string")?;
                if groups.last() != Some(&resource) {
                    groups.push(resource);
                }
                groups
            })
        },
    )?;
    assert_eq!(groups, ORDER);
    Ok(())
}
#[test]
fn filtered_aggregate_reuses_each_direct_resource_diagnostic()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let before = fixture.snapshot()?;
    let _: () = for resource in ORDER {
        let (agent, scope) = match resource {
            "mcp" => ("claude", "project"),
            "statusline" => ("codex", "project"),
            _ => ("claude", "user"),
        };
        let aggregate = fixture.command([
            "doctor", "--agent", agent, "--scope", scope, "--format", "json",
        ])?;
        assert_eq!(fixture.snapshot()?, before);
        let aggregate = json(&aggregate)?;
        let direct = fixture.command([
            "doctor", resource, "--agent", agent, "--scope", scope, "--format", "json",
        ])?;
        assert_eq!(fixture.snapshot()?, before);
        let expected = json(&direct)?;
        let actual = aggregate
            .iter()
            .map(
                |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    let include = {
                        *(diagnostic)
                            .get("resource")
                            .ok_or("missing fixture index resource")?
                            == resource
                    };
                    Ok((include, diagnostic))
                },
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|(include, _)| *include)
            .map(|(_, entry)| entry)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{resource}");
    };
    Ok(())
}
#[test]
fn aggregate_surfaces_every_drift_without_mutation()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fs::remove_file(fixture.home().join(".claude/settings.json"))?;
    fs::remove_file(fixture.home().join(".claude/CLAUDE.md"))?;
    fs::remove_file(fixture.home().join(".claude/skills/alpha"))?;
    fixture.write_home(".claude/commands/deploy.md", "stale\n")?;
    fs::remove_file(fixture.home().join(".claude/rules/aggregate.md"))?;
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"other"}}}"#,
    )?;
    fixture.write_repository(
        ".codex/config.toml",
        "[tui]\nstatus_line = [\"current-dir\", \"model\"]\n",
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["doctor", "--format", "json"])?;
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fixture.snapshot()?, before);
    let drifted = json(&output)?
        .into_iter()
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    *(diagnostic)
                        .get("state")
                        .ok_or("missing fixture index state")?
                        == "drift"
                };
                Ok((include, diagnostic))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .map(
            |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok((*(diagnostic)
                    .get("resource")
                    .ok_or("missing fixture index resource")?)
                .as_str()
                .ok_or("expected JSON string")?
                .to_owned())
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        drifted,
        (*(ORDER).get(1..).ok_or("missing fixture index 1..")?)
            .iter()
            .map(|resource| (*resource).to_owned())
            .collect()
    );
    Ok(())
}
#[test]
fn operational_error_does_not_suppress_later_resources()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let before = fixture.snapshot()?;
    let permissions = fs::metadata(fixture.repository().join("harness/AGENTS.md"))?.permissions();
    set_mode(&fixture.repository().join("harness/AGENTS.md"), 0o000)?;
    let unreadable = fs::metadata(fixture.repository().join("harness/AGENTS.md"))?.permissions();
    let output = fixture.command(["doctor", "--format", "json"])?;
    assert_eq!(
        fs::metadata(fixture.repository().join("harness/AGENTS.md"))?.permissions(),
        unreadable
    );
    fs::set_permissions(fixture.repository().join("harness/AGENTS.md"), permissions)?;
    assert_eq!(fixture.snapshot()?, before);
    assert_eq!(output.status.code(), Some(2));
    let diagnostics = json(&output)?;
    assert!(
        diagnostics
            .iter()
            .map(
                |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok({
                        *(diagnostic)
                            .get("resource")
                            .ok_or("missing fixture index resource")?
                            == "instructions"
                            && *(diagnostic)
                                .get("state")
                                .ok_or("missing fixture index state")?
                                == "error"
                    })
                }
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .any(std::convert::identity)
    );
    assert!(
        diagnostics
            .iter()
            .map(
                |diagnostic| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok({
                        *(diagnostic)
                            .get("resource")
                            .ok_or("missing fixture index resource")?
                            == "statusline"
                            && *(diagnostic)
                                .get("state")
                                .ok_or("missing fixture index state")?
                                == "healthy"
                    })
                }
            )
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .any(std::convert::identity)
    );
    Ok(())
}

#[test]
fn aggregate_preserves_one_fixture_through_healthy_drift_and_fatal_states()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    for (exit_code, resource, state) in [
        (0, "instructions", "healthy"),
        (1, "instructions", "drift"),
        (2, "manifest", "error"),
    ] {
        let before = fixture.snapshot()?;
        for format in ["human", "json"] {
            let output = fixture.command(["doctor", "--format", format])?;
            assert_eq!(fixture.snapshot()?, before, "{state}, {format}");
            assert_eq!(output.status.code(), Some(exit_code), "{state}, {format}");
            assert!(output.stderr.is_empty());
            if format == "json" {
                let diagnostics = json(&output)?;
                assert!(diagnostics.iter().any(|diagnostic| {
                    diagnostic.get("resource").and_then(Value::as_str) == Some(resource)
                        && diagnostic.get("state").and_then(Value::as_str) == Some(state)
                }));
            } else {
                assert!(!output.stdout.is_empty());
            }
        }
        match exit_code {
            0 => fs::remove_file(fixture.home().join(".claude/CLAUDE.md"))?,
            1 => fixture.write_home(".arnes.yaml", "version: [invalid\n")?,
            _ => {}
        }
    }
    Ok(())
}

#[test]
fn undeclared_cursor_aggregate_is_unsupported_instead_of_healthy()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["doctor", "--agent", "cursor", "--format", "json"])?;
    assert_eq!(fixture.snapshot()?, before);
    assert_eq!(output.status.code(), Some(0));
    let diagnostics = json(&output)?;
    let resources = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.get("resource").and_then(Value::as_str) != Some("manifest"))
        .collect::<Vec<_>>();
    assert!(!resources.is_empty());
    assert!(resources.iter().all(|diagnostic| {
        diagnostic.get("state").and_then(Value::as_str) == Some("unsupported")
    }));
    Ok(())
}
