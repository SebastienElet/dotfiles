#![cfg(test)]
mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use support::Fixture;

#[test]
fn native_artifacts_receipts_locks_and_executable_commands_are_reserved_sources() -> TestResult {
    for source in [
        ".mcp.json",
        "..mcp.json.lock",
        ".arnes-mcp-ownership/claude-project-managed.json",
        ".arnes-mcp-ownership/.claude-project-managed.json.lock",
        "alias.md",
        "planned",
    ] {
        let fixture = fixture("claude", "project", "")?;
        let path = if source == "planned" {
            ".arnes-mcp-ownership/claude-project-managed.json"
        } else {
            source
        };
        let declaration = format!(
            "\nprompts:\n  - id: canonical\n    source: {{ root: repository, path: {path} }}\n    includes: []\n    variables: []\n    projections: []\n"
        );
        fixture.write_home(
            ".arnes.yaml",
            &(manifest("claude", "project", "", "first, second") + &declaration),
        )?;
        if source == "alias.md" {
            fixture.write_repository(".mcp.json", "{}")?;
            symlink(
                fixture.repository().join(".mcp.json"),
                fixture.repository().join(path),
            )?;
        } else if source != "planned" {
            fixture.write_repository(
                path,
                if source == ".mcp.json" {
                    "{}"
                } else {
                    "Canonical source\n"
                },
            )?;
        }
        let before = fixture.snapshot()?;
        assert_eq!(
            sync(&fixture, "claude", "project")?.status.code(),
            Some(1),
            "{source}"
        );
        assert_eq!(fixture.snapshot()?, before, "{source}");
    }
    let fixture = fixture("claude", "project", "")?;
    fixture.write_repository("..mcp.json.lock", "#!/bin/sh\nexit 0\n")?;
    fs::set_permissions(
        fixture.repository().join("..mcp.json.lock"),
        fs::Permissions::from_mode(0o700),
    )?;
    fixture.write_home(
        ".arnes.yaml",
        &manifest("claude", "project", "", "first, second")
            .replace("command: safe-mcp", "command: ./..mcp.json.lock"),
    )?;
    let before = fixture.snapshot()?;
    assert_eq!(sync(&fixture, "claude", "project")?.status.code(), Some(1));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn markdown_publication_reserves_other_instruction_sources_and_transitive_includes() -> TestResult {
    for source in [
        ".claude/commands/review.md",
        ".claude/commands/.review.md.lock",
        "indirect.md",
    ] {
        let fixture = Fixture::new()?;
        let prompt = "version: 1\nagents:\n  - { id: claude, scopes: [user, project] }\nresources: []\nprompts:\n  - id: review\n    source: { root: repository, path: review.md }\n    includes: []\n    variables: []\n    projections:\n      - { agent: claude, scope: project, representation: file, destination: { root: repository, path: .claude/commands/review.md } }\n";
        fixture.write_home(".arnes.yaml", prompt)?;
        fixture.write_repository("review.md", "Original prompt\n")?;
        let args = ["sync", "prompts", "--agent", "claude", "--scope", "project"];
        assert!(fixture.command(args)?.status.success());
        if source == "indirect.md" {
            fixture.write_repository(source, "@.claude/commands/review.md\n")?;
        } else if source == ".claude/commands/.review.md.lock" {
            fixture.write_repository(source, "Canonical instruction source\n")?;
        }
        let resource = format!(
            "resources:\n  - id: canonical\n    kind: instructions\n    agent: claude\n    scope: project\n    source: {{ root: repository, path: {source} }}\n    destination: {{ root: repository, path: CLAUDE.md }}"
        );
        fixture.write_home(".arnes.yaml", &prompt.replace("resources: []", &resource))?;
        fixture.write_repository("review.md", "Updated prompt\n")?;
        let before = fixture.snapshot()?;
        assert_eq!(fixture.command(args)?.status.code(), Some(1), "{source}");
        assert_eq!(fixture.snapshot()?, before, "{source}");
    }
    Ok(())
}

#[test]
fn declared_hook_executables_cannot_alias_native_locks_but_separate_commands_are_accepted()
-> TestResult {
    for (command, expected) in [("..mcp.json.lock", 1), ("helper-tool", 0)] {
        let fixture = fixture("claude", "project", "")?;
        fixture.write_repository(command, "#!/bin/sh\nexit 0\n")?;
        fs::set_permissions(
            fixture.repository().join(command),
            fs::Permissions::from_mode(0o700),
        )?;
        fs::create_dir_all(fixture.home().join(".local/bin"))?;
        symlink(
            fixture.repository().join(command),
            fixture.home().join(".local/bin/arnes"),
        )?;
        let hook = "\nhooks:\n  - id: measurement\n    installations:\n      - { agent: claude, scope: user }\n";
        fixture.write_home(
            ".arnes.yaml",
            &(manifest("claude", "project", "", "first, second") + hook),
        )?;
        let before = fixture.snapshot()?;
        assert_eq!(
            sync(&fixture, "claude", "project")?.status.code(),
            Some(expected)
        );
        assert_eq!(
            fs::metadata(fixture.repository().join(command))?
                .permissions()
                .mode()
                & 0o777,
            0o700
        );
        if expected == 1 {
            assert_eq!(fixture.snapshot()?, before);
        }
    }
    Ok(())
}

#[test]
fn claude_project_preferences_and_their_lock_are_reserved_before_registration_publication()
-> TestResult {
    for source in [".claude.json", "..claude.json.lock"] {
        let fixture = fixture("claude", "project", "enabled: false")?;
        fs::remove_file(fixture.home().join(".arnes.yaml"))?;
        let declaration = format!(
            "\nprompts:\n  - id: canonical\n    source: {{ root: repository, path: {source} }}\n    includes: []\n    variables: []\n    projections: []\n"
        );
        fixture.write_repository(
            "home/.arnes.yaml",
            &(manifest("claude", "project", "enabled: false", "first, second") + &declaration),
        )?;
        symlink(
            fixture.repository().join("home/.arnes.yaml"),
            fixture.home().join(".arnes.yaml"),
        )?;
        fixture.write_home(
            source,
            if source == ".claude.json" {
                "{}"
            } else {
                "Canonical source\n"
            },
        )?;
        let before = fixture.snapshot()?;
        let output = fixture.command_from(
            fixture.home(),
            ["sync", "mcp", "--agent", "claude", "--scope", "project"],
        )?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn command_with_path(
    fixture: &Fixture,
    path: &str,
    arguments: &[&str],
) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
    Ok(std::process::Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(arguments)
        .current_dir(fixture.repository())
        .env_clear()
        .env("HOME", fixture.home())
        .env("PATH", path)
        .output()?)
}

fn relative_path_fixture() -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = fixture("claude", "project", "")?;
    fixture.write_repository(
        "safe-mcp",
        "#!/bin/sh\nprintf executed > \"$HOME/should-not-execute\"\n",
    )?;
    fs::set_permissions(
        fixture.repository().join("safe-mcp"),
        fs::Permissions::from_mode(0o700),
    )?;
    let declared = manifest("claude", "project", "", "first, second").replace(
        "scopes: [user, project]",
        "scopes: [user, project]\n    user_config: { model: expected }",
    );
    fixture.write_home(".arnes.yaml", &declared)?;
    Ok(fixture)
}

#[test]
fn relative_and_empty_path_entries_resolve_mcp_commands_without_execution() -> TestResult {
    for path in [".", "", "/missing-arnes-directory:"] {
        let fixture = relative_path_fixture()?;
        let output = command_with_path(
            &fixture,
            path,
            &["sync", "mcp", "--agent", "claude", "--scope", "project"],
        )?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{path}: {}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            command_with_path(
                &fixture,
                path,
                &["doctor", "mcp", "--agent", "claude", "--scope", "project"]
            )?
            .status
            .success()
        );
        assert!(!fixture.home().join("should-not-execute").exists());
    }
    Ok(())
}

#[test]
fn relative_and_empty_path_entries_do_not_block_config_when_mcp_is_unselected() -> TestResult {
    for path in [".", "", "/missing-arnes-directory:"] {
        let fixture = relative_path_fixture()?;
        let output = command_with_path(
            &fixture,
            path,
            &["sync", "config", "--agent", "claude", "--scope", "user"],
        )?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{path}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            command_with_path(
                &fixture,
                path,
                &["doctor", "config", "--agent", "claude", "--scope", "user"]
            )?
            .status
            .success()
        );
        assert!(!fixture.repository().join(".mcp.json").exists());
        assert!(!fixture.home().join("should-not-execute").exists());
    }
    Ok(())
}

#[test]
fn a_command_found_in_a_relative_path_cannot_alias_its_native_lock() -> TestResult {
    for path in [".", ""] {
        let fixture = fixture("claude", "project", "")?;
        fixture.write_repository("..mcp.json.lock", "#!/bin/sh\nexit 0\n")?;
        fs::set_permissions(
            fixture.repository().join("..mcp.json.lock"),
            fs::Permissions::from_mode(0o700),
        )?;
        fixture.write_home(
            ".arnes.yaml",
            &manifest("claude", "project", "", "first, second")
                .replace("command: safe-mcp", "command: ..mcp.json.lock"),
        )?;
        let before = fixture.snapshot()?;
        assert_eq!(
            command_with_path(
                &fixture,
                path,
                &["sync", "mcp", "--agent", "claude", "--scope", "project"]
            )?
            .status
            .code(),
            Some(1)
        );
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn link_creation_cannot_replace_a_planned_source_of_another_resource() -> TestResult {
    let fixture = Fixture::new()?;
    let manifest = "version: 1\nagents:\n  - { id: claude, scopes: [user, project] }\nresources:\n  - id: local-rule\n    kind: rules\n    agent: claude\n    scope: user\n    source: { root: repository, path: rule.md }\n    destination: { root: home, path: .claude/rules/local.md }\n  - id: canonical\n    kind: instructions\n    agent: claude\n    scope: project\n    source: { root: repository, path: .claude/rules/local.md }\n    destination: { root: repository, path: CLAUDE.md }\n";
    fixture.write_repository("home/.arnes.yaml", manifest)?;
    symlink(
        fixture.repository().join("home/.arnes.yaml"),
        fixture.home().join(".arnes.yaml"),
    )?;
    fixture.write_home("rule.md", "Canonical rule\n")?;
    let before = fixture.snapshot()?;
    assert_eq!(
        fixture
            .command_from(
                fixture.home(),
                ["sync", "rules", "--agent", "claude", "--scope", "user"]
            )?
            .status
            .code(),
        Some(1)
    );
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

fn manifest(agent: &str, scope: &str, enabled: &str, args: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: {agent}\n    scopes: [user, project]\nmcp:\n  - name: managed\n    agent: {agent}\n    scope: {scope}\n    command: safe-mcp\n    args: [{args}]\n    environment: [TOKEN]\n    {enabled}\nresources: []\n"
    )
}

fn fixture(
    agent: &str,
    scope: &str,
    enabled: &str,
) -> Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        &manifest(agent, scope, enabled, "first, second"),
    )?;
    fixture.write_home(
        "bin/safe-mcp",
        "#!/bin/sh\nprintf executed > \"$HOME/should-not-execute\"\n",
    )?;
    fs::set_permissions(
        fixture.home().join("bin/safe-mcp"),
        fs::Permissions::from_mode(0o700),
    )?;
    Ok(fixture)
}

fn sync(
    fixture: &Fixture,
    agent: &str,
    scope: &str,
) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
    fixture.command([
        "sync", "mcp", "--agent", agent, "--scope", scope, "--format", "json",
    ])
}

fn relative(agent: &str, scope: &str) -> &'static str {
    match (agent, scope) {
        ("claude", "user") => ".claude.json",
        ("claude", _) => ".mcp.json",
        ("cursor", _) => ".cursor/mcp.json",
        _ => ".codex/config.toml",
    }
}

#[test]
fn creates_native_registrations_for_all_scopes_without_running_servers_and_replays() -> TestResult {
    for (agent, scope, enabled) in [
        ("claude", "user", "enabled: true"),
        ("claude", "project", "enabled: false"),
        ("cursor", "user", ""),
        ("cursor", "project", ""),
        ("codex", "user", "enabled: false"),
        ("codex", "project", "enabled: true"),
    ] {
        let fixture = fixture(agent, scope, enabled)?;
        let original = if agent == "codex" {
            "model = 'private-model'\n[tui]\nstatus_line = ['model']\n[mcp_servers.foreign]\ncommand = 'private-command'\n[unknown]\nvalue = nan\n"
        } else {
            r#"{"future":{"large":9007199254740993,"small":1e-400,"fraction":9007199254740991.1},"mcpServers":{"foreign":{"url":"https://private.invalid","env":{"KEY":"private-secret"}}}}"#
        };
        let root = if scope == "user" {
            fixture.home()
        } else {
            fixture.repository()
        };
        let path = root.join(relative(agent, scope));
        fs::create_dir_all(path.parent().ok_or("parent")?)?;
        fs::write(&path, original)?;
        let output = sync(&fixture, agent, scope)?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(!String::from_utf8_lossy(&output.stdout).contains("private-"));
        let content = fs::read_to_string(&path)?;
        assert!(content.contains("foreign"));
        if agent != "codex" {
            for token in [
                "9007199254740993",
                "1e-400",
                "9007199254740991.1",
                "private-secret",
            ] {
                assert!(content.contains(token), "lost {token}");
            }
            let reference = if agent == "cursor" {
                "${env:TOKEN}"
            } else {
                "${TOKEN}"
            };
            assert!(content.contains(reference));
        }
        assert!(!fixture.home().join("should-not-execute").exists());
        let identity = fs::metadata(&path)?.ino();
        let before = fixture.snapshot()?;
        let output = sync(&fixture, agent, scope)?;
        assert_eq!(output.status.code(), Some(0));
        assert!(String::from_utf8_lossy(&output.stdout).contains("current"));
        assert_eq!(fixture.snapshot()?, before);
        assert_eq!(fs::metadata(path)?.ino(), identity);
        let doctor = fixture.command(["doctor", "mcp", "--agent", agent, "--scope", scope])?;
        assert_eq!(
            doctor.status.code(),
            Some(0),
            "{}",
            String::from_utf8_lossy(&doctor.stdout)
        );
    }
    Ok(())
}

#[test]
fn updates_entries_created_by_arnes_and_refuses_local_divergent_entries() -> TestResult {
    for agent in ["claude", "cursor", "codex"] {
        let fixture = fixture(agent, "user", "")?;
        assert!(sync(&fixture, agent, "user")?.status.success());
        fixture.write_home(".arnes.yaml", &manifest(agent, "user", "", "second, first"))?;
        let output = sync(&fixture, agent, "user")?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(
            fixture
                .command(["doctor", "mcp", "--agent", agent, "--scope", "user"])?
                .status
                .success()
        );
        let path = fixture.home().join(relative(agent, "user"));
        let native = fs::read_to_string(&path)?.replace("safe-mcp", "private-local-command");
        fs::write(&path, &native)?;
        let output = sync(&fixture, agent, "user")?;
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(fs::read_to_string(path)?, native);
        assert!(!String::from_utf8_lossy(&output.stdout).contains("private-local-command"));
    }
    Ok(())
}

#[test]
fn unowned_conforming_entries_remain_current_without_receipt_adoption() -> TestResult {
    let fixture = fixture("claude", "user", "")?;
    let original = r#"{"mcpServers":{"managed":{"command":"safe-mcp","args":["first","second"],"env":{"TOKEN":"${TOKEN}"}}}}"#;
    fixture.write_home(".claude.json", original)?;
    let before = fixture.snapshot()?;
    let output = sync(&fixture, "claude", "user")?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("current"));
    assert_eq!(fixture.snapshot()?, before);
    fixture.write_home(
        ".arnes.yaml",
        &manifest("claude", "user", "", "second, first"),
    )?;
    let before = fixture.snapshot()?;
    assert_eq!(sync(&fixture, "claude", "user")?.status.code(), Some(1));
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn collisions_missing_commands_and_unsupported_enabled_do_not_mutate() -> TestResult {
    for condition in [
        "collision",
        "missing-command",
        "claude-disabled",
        "cursor-enabled",
    ] {
        let enabled = if condition == "claude-disabled" {
            "enabled: false"
        } else if condition == "cursor-enabled" {
            "enabled: true"
        } else {
            ""
        };
        let agent = if condition == "cursor-enabled" {
            "cursor"
        } else {
            "claude"
        };
        let fixture = fixture(agent, "user", enabled)?;
        if condition == "collision" {
            fixture.write_repository(
                ".mcp.json",
                r#"{"mcpServers":{"managed":{"command":"safe-mcp"}}}"#,
            )?;
        }
        if condition == "missing-command" {
            fs::remove_file(fixture.home().join("bin/safe-mcp"))?;
        }
        let before = fixture.snapshot()?;
        let output = sync(&fixture, agent, "user")?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn malformed_values_and_native_links_preserve_files_and_secrets() -> TestResult {
    for original in [
        "[",
        r#"{"mcpServers":false}"#,
        r#"{"mcpServers":{"managed":{"command":42,"env":{"TOKEN":"private-secret"}}}}"#,
        r#"{"mcpServers":{"managed":{"command":"safe-mcp"},"managed":{"command":"private"}}}"#,
    ] {
        let fixture = fixture("claude", "user", "")?;
        fixture.write_home(".claude.json", original)?;
        let before = fixture.snapshot()?;
        let output = sync(&fixture, "claude", "user")?;
        assert!(!output.status.success());
        assert_eq!(fixture.snapshot()?, before);
        assert!(!String::from_utf8_lossy(&output.stdout).contains("private-secret"));
    }
    let fixture = fixture("claude", "user", "")?;
    fixture.write_repository("home/.claude.json", "{}")?;
    symlink(
        fixture.repository().join("home/.claude.json"),
        fixture.home().join(".claude.json"),
    )?;
    let before = fixture.snapshot()?;
    assert!(!sync(&fixture, "claude", "user")?.status.success());
    assert_eq!(fixture.snapshot()?, before);
    Ok(())
}

#[test]
fn receipt_failure_is_explicit_and_never_adopts_the_conforming_result() -> TestResult {
    let fixture = fixture("claude", "user", "")?;
    let receipt_root = fixture.home().join(".arnes-mcp-ownership");
    fs::create_dir(&receipt_root)?;
    fs::set_permissions(&receipt_root, fs::Permissions::from_mode(0o500))?;
    let output = sync(&fixture, "claude", "user")?;
    assert_eq!(output.status.code(), Some(2));
    assert!(
        fixture
            .command(["doctor", "mcp", "--agent", "claude", "--scope", "user"])?
            .status
            .success()
    );
    assert_eq!(
        fs::metadata(&receipt_root)?.permissions().mode() & 0o777,
        0o500
    );
    let output = sync(&fixture, "claude", "user")?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&output.stdout).contains("current"));
    fixture.write_home(
        ".arnes.yaml",
        &manifest("claude", "user", "", "second, first"),
    )?;
    assert_eq!(sync(&fixture, "claude", "user")?.status.code(), Some(1));
    Ok(())
}
