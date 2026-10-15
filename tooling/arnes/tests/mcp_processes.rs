#![cfg(test)]
mod support;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use support::Fixture;

type Result = std::result::Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn fixture(rows: &str) -> std::result::Result<Fixture, Box<dyn std::error::Error + Send + Sync>> {
    let fixture = Fixture::new()?;
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\nresources: []\n",
    )?;
    fixture.write_home(
        ".codex/config.toml",
        "[mcp_servers.context7]\nurl = 'https://example.test/mcp'\n",
    )?;
    fixture.write_home(
        "bin/ps",
        &format!("#!/bin/sh\ncat <<'PROCESSES'\n{rows}\nPROCESSES\n"),
    )?;
    fs::set_permissions(
        fixture.home().join("bin/ps"),
        fs::Permissions::from_mode(0o755),
    )?;
    std::os::unix::fs::symlink("/bin/cat", fixture.home().join("bin/cat"))?;
    Ok(fixture)
}

#[test]
fn agent_mcp_trees_report_instances_memory_age_and_stale_commands_without_secrets() -> Result {
    let fixture = fixture(
        "10 1 100 02:00 /opt/codex app-server\n20 10 1024 01:00 npm exec @upstash/context7-mcp --api-key private-secret\n21 20 2048 00:59 node /cache/@upstash/context7-mcp/dist/index.js --api-key private-secret\n22 10 4096 00:30 node /cache/@upstash/context7-mcp/dist/index.js --api-key private-secret\n30 1 900 01:00 node /cache/@upstash/context7-mcp/dist/index.js --api-key unrelated-secret\n40 10 90 00:20 /bin/sh -c echo context7 private-secret",
    )?;
    let before = fixture.snapshot()?;
    let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    for expected in [
        "session=10",
        "context7",
        "instances=2",
        "processes=3",
        "RAM=7168 KiB",
        "oldest=60s",
        "command differs",
        "codex mcp remove context7",
        "heuristic",
    ] {
        assert!(stdout.contains(expected), "missing {expected}: {stdout}");
    }
    assert!(!stdout.contains("private-secret") && !stdout.contains("unrelated-secret"));
    assert_eq!(fixture.snapshot()?, before);
    assert!(fixture.repository().is_dir());
    Ok(())
}

#[test]
fn declared_stdio_process_is_healthy_and_cycles_terminate() -> Result {
    let fixture = fixture(
        "10 1 100 02:00 /opt/codex app-server\n20 10 1024 01:00 /bin/echo expected\n30 31 0 00:01 /opt/codex\n31 30 0 00:01 /bin/echo expected",
    )?;
    fixture.write_home(".arnes.yaml", "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\nmcp:\n  - { agent: codex, scope: user, name: sample, command: /bin/echo, args: [expected] }\nresources: []\n")?;
    fixture.write_home(
        ".codex/config.toml",
        "[mcp_servers.sample]\ncommand = '/bin/echo'\nargs = ['expected']\n",
    )?;
    let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(stdout.contains("instances=1"), "{stdout}");
    Ok(())
}

#[test]
fn failed_missing_or_malformed_ps_is_an_error_with_redacted_stderr() -> Result {
    for script in [
        "#!/bin/sh\nexit 1\n",
        "#!/bin/sh\nprintf 'private-secret\\n'\n",
        "#!/bin/sh\nexit 0\n",
    ] {
        let fixture = fixture("")?;
        fixture.write_home("bin/ps", script)?;
        let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
        let stdout = String::from_utf8(output.stdout)?;
        assert_eq!(output.status.code(), Some(2), "{stdout}");
        assert!(!stdout.contains("private-secret"));
        fs::remove_file(fixture.home().join("bin/ps"))?;
        let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
        assert_eq!(output.status.code(), Some(2));
    }
    Ok(())
}

#[test]
fn a_different_executable_path_is_stale_even_with_the_same_basename() -> Result {
    let fixture =
        fixture("10 1 100 02:00 /opt/codex app-server\n20 10 1024 01:00 /obsolete/echo expected")?;
    fixture.write_home(".arnes.yaml", "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\nmcp:\n  - { agent: codex, scope: user, name: sample, command: /bin/echo, args: [expected] }\nresources: []\n")?;
    fixture.write_home(
        ".codex/config.toml",
        "[mcp_servers.sample]\ncommand = '/bin/echo'\nargs = ['expected']\n",
    )?;
    let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("command differs"), "{stdout}");
    Ok(())
}

#[test]
fn shared_executables_and_generic_paths_do_not_override_specific_mcp_identity() -> Result {
    let fixture = fixture(
        "10 1 100 02:00 /opt/codex app-server\n20 10 1024 01:00 /bin/echo beta\n21 10 2048 00:30 /users/example/bin/remem",
    )?;
    fixture.write_home(".arnes.yaml", "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\nmcp:\n  - { agent: codex, scope: user, name: alpha, command: /bin/echo, args: [alpha] }\n  - { agent: codex, scope: user, name: beta, command: /bin/echo, args: [beta] }\nresources: []\n")?;
    fixture.write_home(".codex/config.toml", "[mcp_servers.alpha]\ncommand = '/bin/echo'\nargs = ['alpha']\n[mcp_servers.beta]\ncommand = '/bin/echo'\nargs = ['beta']\n[mcp_servers.generic]\ncommand = 'npx'\nargs = ['computer-use-mcp', '/users/example']\n[mcp_servers.remem]\ncommand = '/users/example/bin/remem'\n")?;
    let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("MCP beta session=10: instances=1 processes=1 RAM=1024"),
        "{stdout}"
    );
    assert!(
        stdout.contains("MCP remem session=10: instances=1 processes=1 RAM=2048"),
        "{stdout}"
    );
    assert!(
        !stdout.contains("MCP alpha session=") && !stdout.contains("MCP generic session="),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn process_scope_is_not_inferred_from_a_doctor_filter() -> Result {
    let fixture =
        fixture("10 1 100 02:00 /opt/codex app-server\n20 10 1024 01:00 /bin/echo expected")?;
    fixture.write_home(".arnes.yaml", "version: 1\nagents:\n  - { id: codex, scopes: [user, project] }\nmcp:\n  - { agent: codex, scope: user, name: sample, command: /bin/echo, args: [expected] }\nresources: []\n")?;
    fixture.write_home(
        ".codex/config.toml",
        "[mcp_servers.sample]\ncommand = '/bin/echo'\nargs = ['expected']\n",
    )?;
    let output = fixture.command(["doctor", "mcp", "--scope", "project", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    assert!(stdout.contains("scope=unavailable"), "{stdout}");
    Ok(())
}

#[test]
fn owner_configuration_takes_priority_over_another_agents_exact_launch() -> Result {
    let fixture = fixture(
        "10 1 100 02:00 /opt/codex app-server\n20 10 1024 01:00 npm exec @upstash/context7-mcp\n21 20 2048 00:30 node /cache/context7-mcp.js",
    )?;
    fixture.write_home(
        ".claude.json",
        r#"{"mcpServers":{"Context7":{"command":"node","args":["/cache/context7-mcp.js"]}}}"#,
    )?;
    let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
    let stdout = String::from_utf8(output.stdout)?;
    assert!(
        stdout.contains("MCP context7 session=10: instances=1 processes=2 RAM=3072"),
        "{stdout}"
    );
    assert!(!stdout.contains("MCP Context7 session="), "{stdout}");
    Ok(())
}

#[test]
fn ordinary_agent_children_with_mcp_named_arguments_are_not_servers() -> Result {
    for (config, exit_code) in [
        ("", 0),
        (
            "[mcp_servers.context7]\nurl = 'https://example.test/mcp'\n",
            1,
        ),
    ] {
        let fixture = fixture(
            "10 1 100 02:00 /opt/codex app-server\n20 10 1024 01:00 /usr/bin/tail -f mcp.log\n21 10 2048 00:30 /tools/cargo test /work/mcp/tests\n22 10 4096 00:20 /usr/bin/cat /cache/context7-mcp/dist/index.js\n23 10 1024 00:10 /usr/bin/tail -f context7.log",
        )?;
        fixture.write_home(".codex/config.toml", config)?;
        let before = fixture.snapshot()?;
        let output = fixture.command(["doctor", "mcp", "--format", "json"])?;
        let stdout = String::from_utf8(output.stdout)?;
        assert!(!stdout.contains("session=10"), "{stdout}");
        assert_eq!(output.status.code(), Some(exit_code), "{stdout}");
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}
