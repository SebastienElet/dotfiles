#![cfg(test)]
mod support;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use support::Fixture;
const ALL_AGENTS: &str = "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [user, project]
  - id: codex
    scopes: [user, project]
resources: []
";
const CLAUDE_USER: &str = "version: 1
agents:
  - id: claude
    scopes: [user]
resources: []
";
fn configured_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", ALL_AGENTS)?;
    for path in [".claude/settings.json", ".cursor/cli-config.json"] {
        fixture.write_home(path, r#"{"unknown": true}"#)?;
    }
    fixture.write_home(".codex/config.toml", "unknown = true\n")?;
    for path in [".claude/settings.json", ".cursor/cli.json"] {
        fixture.write_repository(path, r#"{"unknown": true}"#)?;
    }
    fixture.write_repository(".codex/config.toml", "unknown = true\n")?;
    Ok(fixture)
}
fn run(
    fixture: &Fixture,
    args: &[&str],
) -> Result<(i32, String, String), Box<dyn std::error::Error + Send + Sync>> {
    let output = fixture.command(args)?;
    Ok((
        output
            .status
            .code()
            .ok_or("child process exited without a status code")?,
        String::from_utf8(output.stdout)?,
        String::from_utf8(output.stderr)?,
    ))
}
#[test]
fn user_scope_is_default_and_accepts_unknown_keys()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "config", "-v"])?;
    assert_eq!(code, 0);
    assert_eq!(stdout.matches("healthy config:").count(), 3);
    assert!(stderr.is_empty());
    let _: () = for expected in [
        "healthy config: claude user",
        "healthy config: cursor user",
        "healthy config: codex user",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    };
    Ok(())
}
#[test]
fn agent_and_scope_filters_isolate_selected_configurations()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    fixture.write_repository(".cursor/cli.json", "[")?;
    for (args, expected_lines, expected) in [
        (
            vec![
                "doctor", "config", "--agent", "codex", "--scope", "user", "-v",
            ],
            1,
            "healthy config: codex user",
        ),
        (
            vec!["doctor", "config", "--agent", "claude", "-v"],
            1,
            "healthy config: claude user",
        ),
    ] {
        let (code, stdout, stderr) = run(&fixture, &args)?;
        assert_eq!(code, 0);
        assert_eq!(stdout.matches("healthy config:").count(), expected_lines);
        assert!(stdout.contains(expected), "{stdout}");
        assert!(!stdout.contains("cursor"), "{stdout}");
        assert!(stderr.is_empty());
    }
    let (code, stdout, _) = run(&fixture, &["doctor", "config", "--scope", "project", "-v"])?;
    assert_eq!(code, 2);
    assert_eq!(stdout.matches(" config:").count(), 3);
    assert!(stdout.contains("error config: cursor project"));
    assert!(!stdout.contains("cursor user"));
    Ok(())
}
#[test]
fn missing_roots_and_files_are_drift() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", ALL_AGENTS)?;
    let (code, stdout, _) = run(&fixture, &["doctor", "config"])?;
    assert_eq!(code, 1);
    assert_eq!(stdout.matches("root ").count(), 3);
    assert_eq!(stdout.matches("is missing").count(), 3);
    for path in [".claude", ".cursor", ".codex"] {
        fs::create_dir_all(fixture.home().join(path))?;
        fs::create_dir_all(fixture.repository().join(path))?;
    }
    let (code, stdout, _) = run(&fixture, &["doctor", "config"])?;
    assert_eq!(code, 1);
    assert_eq!(stdout.matches("file ").count(), 3);
    assert_eq!(stdout.matches("is missing").count(), 3);
    Ok(())
}
#[test]
fn unreadable_and_invalid_paths_are_errors() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = configured_fixture()?;
    set_mode(&fixture.home().join(".claude"), 0o000)?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "config", "--agent", "claude", "--scope", "user"],
    )?;
    set_mode(&fixture.home().join(".claude"), 0o700)?;
    assert_eq!(code, 2);
    assert!(
        stdout.contains("root ~/.claude could not be read"),
        "{stdout}"
    );
    set_mode(&fixture.home().join(".cursor/cli-config.json"), 0o000)?;
    let (code, stdout, _) = run(
        &fixture,
        &["doctor", "config", "--agent", "cursor", "--scope", "user"],
    )?;
    set_mode(&fixture.home().join(".cursor/cli-config.json"), 0o600)?;
    assert_eq!(code, 2);
    assert!(stdout.contains("file ~/.cursor/cli-config.json could not be read"));
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", CLAUDE_USER)?;
    fixture.write_home(".claude", "not a directory")?;
    let (code, stdout, _) = run(&fixture, &["doctor", "config"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("root ~/.claude is not a directory"));
    Ok(())
}
#[test]
fn malformed_formats_and_wrong_json_roots_are_errors()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (agent, scope, path, contents, expected) in [
        (
            "claude",
            "user",
            ".claude/settings.json",
            "{",
            "malformed JSON",
        ),
        (
            "cursor",
            "user",
            ".cursor/cli-config.json",
            "[]",
            "top-level JSON object",
        ),
        (
            "codex",
            "user",
            ".codex/config.toml",
            "value = [",
            "malformed TOML",
        ),
    ] {
        let fixture = configured_fixture()?;
        fixture.write_home(path, contents)?;
        let (code, stdout, stderr) = run(
            &fixture,
            &["doctor", "config", "--agent", agent, "--scope", scope],
        )?;
        assert_eq!(code, 2);
        assert!(stdout.contains(expected), "{stdout}");
        assert!(stderr.is_empty());
    };
    Ok(())
}
#[test]
fn non_files_and_non_utf8_files_are_errors() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", CLAUDE_USER)?;
    fs::create_dir_all(fixture.home().join(".claude/settings.json"))?;
    let (code, stdout, _) = run(&fixture, &["doctor", "config"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("file ~/.claude/settings.json is not a file"));
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", CLAUDE_USER)?;
    fs::create_dir_all(fixture.home().join(".claude"))?;
    fs::write(fixture.home().join(".claude/settings.json"), [0xff])?;
    let (code, stdout, _) = run(&fixture, &["doctor", "config"])?;
    assert_eq!(code, 2);
    assert!(stdout.contains("file ~/.claude/settings.json could not be read"));
    Ok(())
}
#[test]
fn undeclared_filtered_combinations_are_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", CLAUDE_USER)?;
    let (code, stdout, stderr) = run(
        &fixture,
        &["doctor", "config", "--agent", "codex", "--scope", "project"],
    )?;
    assert_eq!(code, 0);
    assert_eq!(
        stdout,
        "Config · project scope · codex agent\n✓ 0 healthy\n! 1 unsupported (non-blocking)\n\nunsupported config: codex project configuration is not declared in the manifest\n"
    );
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn empty_manifests_are_explicitly_unsupported()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for manifest in [
        "version: 1\nagents: []\nresources: []\n",
        "version: 1\nagents:\n  - id: claude\n    scopes: []\nresources: []\n",
    ] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", manifest)?;
        let (code, stdout, stderr) = run(&fixture, &["doctor", "config"])?;
        assert_eq!(code, 0);
        assert_eq!(
            stdout,
            "Config · user scope\n✓ 0 healthy\n! 1 unsupported (non-blocking)\n\nunsupported config: user configuration scope is not declared in the manifest\n"
        );
        assert!(stderr.is_empty());
    };
    Ok(())
}
#[test]
fn manifest_failures_fail_closed_as_config_errors()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    let (code, stdout, stderr) = run(&fixture, &["doctor", "config"])?;
    assert_eq!(code, 2);
    assert_eq!(
        stdout,
        "Config · user scope\n✓ 0 healthy\n\nerror config: manifest: .arnes.yaml was not found\n"
    );
    assert!(stderr.is_empty());
    Ok(())
}
#[test]
fn config_doctor_is_read_only() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let before = fixture.snapshot()?;
    let (code, _, _) = run(&fixture, &["doctor", "config"])?;
    assert_eq!(code, 0);
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}
#[test]
fn default_doctor_reuses_filtered_config_diagnostics()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = configured_fixture()?;
    let (_, direct, _) = run(
        &fixture,
        &[
            "doctor", "config", "--agent", "codex", "--scope", "user", "--format", "json",
        ],
    )?;
    let (_, aggregate, _) = run(
        &fixture,
        &[
            "doctor", "--agent", "codex", "--scope", "user", "--format", "json",
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
                        == "config"
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
fn set_mode(
    path: &std::path::Path,
    mode: u32,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions)?;
    Ok(())
}
