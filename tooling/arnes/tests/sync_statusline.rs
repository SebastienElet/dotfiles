#![cfg(test)]
mod support;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt, symlink};
use support::Fixture;

type TestResult = Result<(), Box<dyn std::error::Error + Send + Sync>>;

fn manifest(scope: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: codex\n    scopes: [user, project]\n  - id: claude\n    scopes: [user]\nstatuslines:\n  - {{ agent: codex, scope: {scope}, items: [model, current-dir] }}\nresources: []\n"
    )
}

fn synchronize(
    fixture: &Fixture,
    agent: &str,
    scope: &str,
) -> Result<std::process::Output, Box<dyn std::error::Error + Send + Sync>> {
    fixture.command([
        "sync",
        "statusline",
        "--agent",
        agent,
        "--scope",
        scope,
        "--format",
        "json",
    ])
}

#[test]
fn synchronizes_ordered_items_preserves_native_values_and_replays_without_rewrite() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest("user"))?;
    let original = "model = \"private-model\"\n[features]\nhooks = true\n[tui]\nanimations = false\nstatus_line = [\"current-dir\", \"model\"]\n[mcp_servers.private]\ncommand = \"private-command\"\n";
    fixture.write_home(".codex/config.toml", original)?;
    fixture.write_repository(".codex/config.toml", "model = \"project-model\"\n")?;
    let path = fixture.home().join(".codex/config.toml");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o640))?;
    let output = synchronize(&fixture, "codex", "user")?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered = String::from_utf8(output.stdout)?;
    assert!(rendered.contains("applied"));
    assert!(!rendered.contains("private-model"));
    assert!(!rendered.contains("private-command"));
    let actual: toml::Value = toml::from_str(&fs::read_to_string(&path)?)?;
    let mut expected: toml::Value = toml::from_str(original)?;
    let tui = expected
        .get_mut("tui")
        .and_then(toml::Value::as_table_mut)
        .ok_or("missing tui")?;
    tui.insert(
        "status_line".to_owned(),
        toml::Value::Array(vec![
            toml::Value::String("model".to_owned()),
            toml::Value::String("current-dir".to_owned()),
        ]),
    );
    assert_eq!(actual, expected);
    assert_eq!(fs::metadata(&path)?.permissions().mode() & 0o777, 0o640);
    assert_eq!(
        fs::read_to_string(fixture.repository().join(".codex/config.toml"))?,
        "model = \"project-model\"\n"
    );
    let before = fs::metadata(&path)?;
    let bytes = fs::read(&path)?;
    assert_eq!(
        synchronize(&fixture, "codex", "user")?.status.code(),
        Some(0)
    );
    assert_eq!(fs::read(&path)?, bytes);
    assert_eq!(fs::metadata(&path)?.ino(), before.ino());
    assert_eq!(fs::metadata(&path)?.modified()?, before.modified()?);
    assert_eq!(
        fixture
            .command([
                "doctor",
                "statusline",
                "--agent",
                "codex",
                "--scope",
                "user"
            ])?
            .status
            .code(),
        Some(0)
    );
    Ok(())
}

#[test]
fn creates_missing_configuration_only_in_the_selected_project_scope() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest("project"))?;
    fixture.write_home(".codex/config.toml", "model = \"keep-user\"\n")?;
    assert_eq!(
        synchronize(&fixture, "codex", "project")?.status.code(),
        Some(0)
    );
    assert_eq!(
        fs::read_to_string(fixture.home().join(".codex/config.toml"))?,
        "model = \"keep-user\"\n"
    );
    assert_eq!(
        fixture
            .command([
                "doctor",
                "statusline",
                "--agent",
                "codex",
                "--scope",
                "project"
            ])?
            .status
            .code(),
        Some(0)
    );
    Ok(())
}

#[test]
fn malformed_or_incompatible_native_configuration_is_preserved_without_exposing_values()
-> TestResult {
    for original in [
        "token = \"private-secret\"\ninvalid = [",
        "tui = \"private-secret\"\n",
        "[tui]\nstatus_line = [42]\n",
        "[tui]\nstatus_line = true\n",
    ] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", &manifest("user"))?;
        fixture.write_home(".codex/config.toml", original)?;
        let output = synchronize(&fixture, "codex", "user")?;
        assert_ne!(output.status.code(), Some(0));
        assert_eq!(
            fs::read_to_string(fixture.home().join(".codex/config.toml"))?,
            original
        );
        assert!(!String::from_utf8(output.stdout)?.contains("private-secret"));
        assert!(!String::from_utf8(output.stderr)?.contains("private-secret"));
    }
    Ok(())
}

#[test]
fn symlink_hardlink_and_parent_links_cannot_mutate_canonical_or_foreign_files() -> TestResult {
    for kind in ["symlink", "hardlink", "parent"] {
        let fixture = Fixture::new()?;
        fixture.write_home(".arnes.yaml", &manifest("user"))?;
        fixture.write_repository("source/config.toml", "model = \"keep-source\"\n")?;
        let source = fixture.repository().join("source/config.toml");
        let directory = fixture.home().join(".codex");
        if kind == "parent" {
            symlink(fixture.repository().join("source"), &directory)?;
        } else {
            fs::create_dir(&directory)?;
            if kind == "symlink" {
                symlink(&source, directory.join("config.toml"))?;
            } else {
                fs::hard_link(&source, directory.join("config.toml"))?;
            }
        }
        assert_ne!(
            synchronize(&fixture, "codex", "user")?.status.code(),
            Some(0)
        );
        assert_eq!(fs::read_to_string(&source)?, "model = \"keep-source\"\n");
    }
    Ok(())
}

#[test]
fn empty_and_unsupported_selections_are_explicit_without_filesystem_mutation() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest("user"))?;
    for (agent, scope, expected) in [
        ("claude", "user", "unsupported"),
        ("codex", "project", "empty"),
    ] {
        let before = fixture.snapshot()?;
        let output = synchronize(&fixture, agent, scope)?;
        assert_ne!(output.status.code(), Some(0));
        assert!(String::from_utf8(output.stdout)?.contains(expected));
        assert_eq!(fixture.snapshot()?, before);
    }
    Ok(())
}

#[test]
fn a_user_scope_inside_the_deployment_repository_preserves_canonical_sources() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_repository("home/.arnes.yaml", &manifest("user"))?;
    fixture.write_repository("home/.codex/config.toml", "model = \"keep-source\"\n")?;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_arnes"))
        .current_dir(fixture.repository())
        .env("HOME", fixture.repository().join("home"))
        .args(["sync", "statusline", "--agent", "codex", "--scope", "user"])
        .output()?;
    assert_ne!(output.status.code(), Some(0));
    assert_eq!(
        fs::read_to_string(fixture.repository().join("home/.codex/config.toml"))?,
        "model = \"keep-source\"\n"
    );
    Ok(())
}

#[test]
fn a_distinct_deployment_manifest_does_not_authorize_writing_current_repository_sources()
-> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_repository(
        "harness/skills/alpha/.codex/config.toml",
        "model = \"keep-current-source\"\n",
    )?;
    let source_home = fixture.repository().join("harness/skills/alpha");
    let deployment = tempfile::tempdir()?;
    fs::create_dir(deployment.path().join("home"))?;
    let deployed_manifest = deployment.path().join("home/.arnes.yaml");
    fs::write(&deployed_manifest, manifest("user"))?;
    symlink(deployed_manifest, source_home.join(".arnes.yaml"))?;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_arnes"))
        .current_dir(fixture.repository())
        .env("HOME", &source_home)
        .args(["sync", "statusline", "--agent", "codex", "--scope", "user"])
        .output()?;
    assert_ne!(output.status.code(), Some(0));
    assert_eq!(
        fs::read_to_string(source_home.join(".codex/config.toml"))?,
        "model = \"keep-current-source\"\n"
    );
    Ok(())
}

#[test]
fn a_conforming_statusline_with_an_unmanaged_nan_value_is_not_rewritten() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest("user"))?;
    let original = "metric=nan\n[tui]\nstatus_line=[\"model\",\"current-dir\"]\n";
    fixture.write_home(".codex/config.toml", original)?;
    let path = fixture.home().join(".codex/config.toml");
    let before = fs::metadata(&path)?;
    let output = synchronize(&fixture, "codex", "user")?;
    assert_eq!(output.status.code(), Some(0));
    assert!(String::from_utf8(output.stdout)?.contains("current"));
    assert_eq!(fs::read_to_string(&path)?, original);
    assert_eq!(fs::metadata(&path)?.ino(), before.ino());
    assert_eq!(fs::metadata(&path)?.modified()?, before.modified()?);
    Ok(())
}

#[test]
fn publication_failure_preserves_existing_configuration() -> TestResult {
    let fixture = Fixture::new()?;
    fixture.write_home(".arnes.yaml", &manifest("user"))?;
    fixture.write_home(".codex/config.toml", "model = \"keep\"\n")?;
    let directory = fixture.home().join(".codex");
    fs::create_dir(directory.join(".config.toml.lock"))?;
    let output = synchronize(&fixture, "codex", "user")?;
    assert_ne!(output.status.code(), Some(0));
    assert_eq!(
        fs::read_to_string(directory.join("config.toml"))?,
        "model = \"keep\"\n"
    );
    Ok(())
}
