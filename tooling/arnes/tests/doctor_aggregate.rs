#[path = "support/doctor_aggregate.rs"]
mod aggregate_support;
mod support;

use aggregate_support::{ORDER, configured_fixture, set_mode};
use serde_json::Value;
use std::fs;

fn json(output: &std::process::Output) -> Vec<Value> {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn healthy_aggregate_is_deterministic_ordered_and_read_only() {
    let fixture = configured_fixture();
    let before = fixture.snapshot();
    let human = fixture.command(["doctor", "-v"]);
    let repeated_human = fixture.command(["doctor", "-v"]);
    let structured = fixture.command(["doctor", "--format", "json"]);
    let repeated_structured = fixture.command(["doctor", "--format", "json"]);

    assert_eq!(
        human.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&human.stdout)
    );
    assert_eq!(human.stdout, repeated_human.stdout);
    assert_eq!(structured.status.code(), Some(0));
    assert_eq!(structured.stdout, repeated_structured.stdout);
    assert_eq!(fixture.snapshot(), before);
    let human = String::from_utf8(human.stdout).unwrap();
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
    .map(|heading| {
        human
            .find(heading)
            .unwrap_or_else(|| panic!("missing {heading:?}: {human}"))
    });
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
    let diagnostics = json(&structured);
    for resource in ORDER {
        assert!(
            diagnostics.iter().any(|diagnostic| {
                diagnostic["resource"] == resource && diagnostic["state"] == "healthy"
            }),
            "missing healthy {resource} diagnostic"
        );
    }
    let groups = diagnostics
        .iter()
        .fold(Vec::new(), |mut groups, diagnostic| {
            let resource = diagnostic["resource"].as_str().unwrap();
            if groups.last() != Some(&resource) {
                groups.push(resource);
            }
            groups
        });
    assert_eq!(groups, ORDER);
}

#[test]
fn filtered_aggregate_reuses_each_direct_resource_diagnostic() {
    let fixture = configured_fixture();

    for resource in ORDER {
        let (agent, scope) = match resource {
            "mcp" => ("claude", "project"),
            "statusline" => ("codex", "project"),
            _ => ("claude", "user"),
        };
        let aggregate = fixture.command([
            "doctor", "--agent", agent, "--scope", scope, "--format", "json",
        ]);
        let aggregate = json(&aggregate);
        let direct = fixture.command([
            "doctor", resource, "--agent", agent, "--scope", scope, "--format", "json",
        ]);
        let expected = json(&direct);
        let actual = aggregate
            .iter()
            .filter(|diagnostic| diagnostic["resource"] == resource)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{resource}");
    }
}

#[test]
fn aggregate_surfaces_every_drift_without_mutation() {
    let fixture = configured_fixture();
    fs::remove_file(fixture.home().join(".claude/settings.json")).unwrap();
    fs::remove_file(fixture.home().join(".claude/CLAUDE.md")).unwrap();
    fs::remove_file(fixture.home().join(".claude/skills/alpha")).unwrap();
    fixture.write_home(".claude/commands/deploy.md", "stale\n");
    fs::remove_file(fixture.home().join(".claude/rules/aggregate.md")).unwrap();
    fixture.write_repository(
        ".mcp.json",
        r#"{"mcpServers":{"managed":{"command":"other"}}}"#,
    );
    fixture.write_repository(
        ".codex/config.toml",
        "[tui]\nstatus_line = [\"current-dir\", \"model\"]\n",
    );
    let before = fixture.snapshot();

    let output = fixture.command(["doctor", "--format", "json"]);

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fixture.snapshot(), before);
    let drifted = json(&output)
        .into_iter()
        .filter(|diagnostic| diagnostic["state"] == "drift")
        .map(|diagnostic| diagnostic["resource"].as_str().unwrap().to_owned())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        drifted,
        ORDER[1..]
            .iter()
            .map(|resource| (*resource).to_owned())
            .collect()
    );
}

#[test]
fn operational_error_does_not_suppress_later_resources() {
    let fixture = configured_fixture();
    set_mode(&fixture.repository().join("harness/AGENTS.md"), 0o000);

    let output = fixture.command(["doctor", "--format", "json"]);

    set_mode(&fixture.repository().join("harness/AGENTS.md"), 0o600);
    assert_eq!(output.status.code(), Some(2));
    let diagnostics = json(&output);
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["resource"] == "instructions" && diagnostic["state"] == "error"
    }));
    assert!(diagnostics.iter().any(|diagnostic| {
        diagnostic["resource"] == "statusline" && diagnostic["state"] == "healthy"
    }));
}
