mod support;

use std::fs;
use support::Fixture;

fn manifest(scope: &str, items: &str) -> String {
    format!(
        "version: 1\nagents:\n  - id: claude\n    scopes: [user, project]\n  - id: cursor\n    scopes: [user, project]\n  - id: codex\n    scopes: [user, project]\nstatuslines:\n  - {{ agent: codex, scope: {scope}, items: [{items}] }}\nresources: []\n"
    )
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn matching_user_statusline_is_healthy_without_mutation() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        &manifest("user", "model-with-reasoning, current-dir"),
    );
    fixture.write_home(
        ".codex/config.toml",
        "secret = \"not-rendered\"\n[tui]\nstatus_line = [\"model-with-reasoning\", \"current-dir\"]\n",
    );
    let before = fixture.snapshot();

    let output = fixture.command([
        "doctor",
        "statusline",
        "--agent",
        "codex",
        "--scope",
        "user",
        "-v",
    ]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("healthy statusline: codex user"));
    assert!(!stdout(&output).contains("not-rendered"));
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn ordered_statusline_mismatch_is_drift() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        &manifest("user", "model-with-reasoning, current-dir"),
    );
    fixture.write_home(
        ".codex/config.toml",
        "[tui]\nstatus_line = [\"current-dir\", \"model-with-reasoning\"]\n",
    );

    let output = fixture.command(["doctor", "statusline", "--agent", "codex"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("ordered items differ"));
}

#[test]
fn missing_statusline_is_drift() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));

    let output = fixture.command(["doctor", "statusline"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("configuration is missing"));
}

#[test]
fn unsupported_agents_and_undeclared_scopes_are_silent() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));

    for args in [
        [
            "doctor",
            "statusline",
            "--agent",
            "claude",
            "--format",
            "json",
        ],
        [
            "doctor",
            "statusline",
            "--agent",
            "cursor",
            "--format",
            "json",
        ],
        [
            "doctor",
            "statusline",
            "--scope",
            "project",
            "--format",
            "json",
        ],
    ] {
        let output = fixture.command(args);
        assert_eq!(output.status.code(), Some(0));
        assert_eq!(stdout(&output), "[]\n");
    }
}

#[test]
fn malformed_or_wrongly_typed_codex_configuration_is_error() {
    for (contents, expected) in [
        ("not = [", "Codex configuration is malformed"),
        ("tui = true", "tui must be a table"),
        (
            "[tui]\nstatus_line = true",
            "tui.status_line must be an array of strings",
        ),
        (
            "[tui]\nstatus_line = [1]",
            "tui.status_line must be an array of strings",
        ),
    ] {
        let fixture = Fixture::new();
        fixture.write_home(".arnes.yaml", &manifest("user", "model"));
        fixture.write_home(".codex/config.toml", contents);

        let output = fixture.command(["doctor", "statusline"]);

        assert_eq!(output.status.code(), Some(2));
        assert!(stdout(&output).contains(expected));
    }
}

#[test]
fn project_scope_reads_only_project_codex_configuration() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("project", "project-item"));
    fixture.write_home(
        ".codex/config.toml",
        "[tui]\nstatus_line = [\"wrong-user-item\"]\n",
    );
    let project_configuration = fixture.repository().join(".codex/config.toml");
    fs::create_dir_all(project_configuration.parent().unwrap()).unwrap();
    fs::write(
        project_configuration,
        "[tui]\nstatus_line = [\"project-item\"]\n",
    )
    .unwrap();

    let output = fixture.command(["doctor", "statusline", "--scope", "project", "-v"]);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout(&output).contains("healthy statusline: codex project"));
}

#[test]
fn unscoped_statusline_diagnoses_all_declared_scopes_without_a_scope_qualifier() {
    let fixture = Fixture::new();
    fixture.write_home(
        ".arnes.yaml",
        "version: 1\nagents:\n  - id: codex\n    scopes: [user, project]\nstatuslines:\n  - { agent: codex, scope: user, items: [model] }\n  - { agent: codex, scope: project, items: [current-dir] }\nresources: []\n",
    );
    fixture.write_home(".codex/config.toml", "[tui]\nstatus_line = [\"model\"]\n");
    fixture.write_repository(
        ".codex/config.toml",
        "[tui]\nstatus_line = [\"current-dir\"]\n",
    );

    let output = fixture.command(["doctor", "statusline", "-v"]);
    let rendered = stdout(&output);

    assert_eq!(output.status.code(), Some(0));
    assert!(rendered.contains("healthy statusline: codex user"));
    assert!(rendered.contains("healthy statusline: codex project"));
    assert!(!rendered.contains("Statusline · user scope"));
}

#[test]
fn missing_statusline_key_is_drift() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));
    fixture.write_home(".codex/config.toml", "[tui]\nanimations = false\n");

    let output = fixture.command(["doctor", "statusline"]);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout(&output).contains("configuration is missing"));
}

#[test]
fn configuration_symlink_cannot_escape_scope_root() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));
    let outside = tempfile::tempdir().unwrap();
    let external = outside.path().join("config.toml");
    fs::write(
        &external,
        "secret = \"outside\"\n[tui]\nstatus_line = [\"model\"]\n",
    )
    .unwrap();
    fs::create_dir_all(fixture.home().join(".codex")).unwrap();
    symlink(&external, fixture.home().join(".codex/config.toml")).unwrap();

    let output = fixture.command(["doctor", "statusline"]);

    assert_eq!(output.status.code(), Some(2));
    assert!(stdout(&output).contains("escapes its scope root"));
    assert!(!stdout(&output).contains("outside"));
}

#[test]
fn default_doctor_reuses_filtered_statusline_diagnostics() {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", &manifest("user", "model"));
    fixture.write_home(".codex/config.toml", "[tui]\nstatus_line = [\"model\"]\n");
    let direct = fixture.command([
        "doctor",
        "statusline",
        "--agent",
        "codex",
        "--scope",
        "user",
        "--format",
        "json",
    ]);
    let aggregate = fixture.command([
        "doctor", "--agent", "codex", "--scope", "user", "--format", "json",
    ]);
    let direct: Vec<serde_json::Value> = serde_json::from_str(&stdout(&direct)).unwrap();
    let aggregate: Vec<serde_json::Value> = serde_json::from_str(&stdout(&aggregate)).unwrap();
    let aggregate = aggregate
        .into_iter()
        .filter(|diagnostic| diagnostic["resource"] == "statusline")
        .collect::<Vec<_>>();

    assert_eq!(aggregate, direct);
}
