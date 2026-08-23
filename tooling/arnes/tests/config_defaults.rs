mod support;

use support::Fixture;

const MANIFEST: &str = "version: 1
agents:
  - id: claude
    scopes: [user, project]
    user_config:
      model: opus[1m]
      effort: high
      auto_compact_window: 600000
  - id: cursor
    scopes: [user, project]
    user_config:
      model: grok-4.6
      max_mode: false
  - id: codex
    scopes: [user, project]
    user_config:
      model: gpt-5.6-sol
      effort: medium
      context_window: 270000
resources: []
";

fn configured_fixture() -> Fixture {
    let fixture = Fixture::new();
    fixture.write_home(".arnes.yaml", MANIFEST);
    fixture.write_home(
        ".claude/settings.json",
        r#"{
            "model": "opus[1m]",
            "effortLevel": "high",
            "autoCompactWindow": 600000,
            "unknown": true
        }"#,
    );
    fixture.write_home(
        ".cursor/cli-config.json",
        r#"{
            "model": {"modelId": "grok-4.6", "displayName": "Cursor Grok 4.6"},
            "maxMode": false,
            "unknown": true
        }"#,
    );
    fixture.write_home(
        ".codex/config.toml",
        "model = \"gpt-5.6-sol\"\nmodel_reasoning_effort = \"medium\"\nmodel_context_window = 270000\nunknown = true\n",
    );
    for path in [".claude/settings.json", ".cursor/cli.json"] {
        fixture.write_repository(path, r#"{"unknown": true}"#);
    }
    fixture.write_repository(".codex/config.toml", "unknown = true\n");
    fixture
}

fn run(fixture: &Fixture, args: &[&str]) -> (i32, String) {
    let output = fixture.command(args);
    (
        output.status.code().unwrap(),
        String::from_utf8(output.stdout).unwrap(),
    )
}

#[test]
fn native_user_settings_satisfy_manifest_defaults_as_subsets() {
    let fixture = configured_fixture();
    let before = fixture.snapshot();
    let (code, stdout) = run(&fixture, &["doctor", "config", "-v"]);

    assert_eq!(code, 0);
    assert_eq!(stdout.matches("healthy config:").count(), 3);
    assert!(fixture.home().is_dir());
    assert!(fixture.repository().is_dir());
    assert_eq!(fixture.snapshot(), before);
}

#[test]
fn missing_and_different_defaults_are_drift() {
    let fixture = configured_fixture();
    fixture.write_home(
        ".claude/settings.json",
        r#"{"model":"sonnet","effortLevel":"xhigh"}"#,
    );
    let (code, stdout) = run(&fixture, &["doctor", "config", "--agent", "claude", "-v"]);

    assert_eq!(code, 1);
    assert!(stdout.contains(r#"model is "sonnet" (expected "opus[1m]")"#));
    assert!(stdout.contains(r#"effortLevel is "xhigh" (expected "high")"#));
    assert!(stdout.contains("autoCompactWindow is missing (expected 600000)"));
}

#[test]
fn cursor_model_id_is_compared_without_requiring_managed_metadata() {
    let fixture = configured_fixture();
    fixture.write_home(
        ".cursor/cli-config.json",
        r#"{"model":{"modelId":"auto"},"maxMode":true}"#,
    );
    let (code, stdout) = run(&fixture, &["doctor", "config", "--agent", "cursor", "-v"]);

    assert_eq!(code, 1);
    assert!(stdout.contains(r#"model.modelId is "auto" (expected "grok-4.6")"#));
    assert!(stdout.contains("maxMode is true (expected false)"));
}

#[test]
fn codex_defaults_use_toml_key_names() {
    let fixture = configured_fixture();
    fixture.write_home(
        ".codex/config.toml",
        "model = \"gpt-5.6-sol\"\nmodel_reasoning_effort = \"medium\"\n",
    );
    let (code, stdout) = run(&fixture, &["doctor", "config", "--agent", "codex", "-v"]);

    assert_eq!(code, 1);
    assert!(stdout.contains("model_context_window is missing (expected 270000)"));
    assert!(!stdout.contains("model_auto_compact_token_limit"));
}

#[test]
fn project_configurations_do_not_inherit_user_defaults() {
    let fixture = configured_fixture();
    let (code, stdout) = run(&fixture, &["doctor", "config", "--scope", "project", "-v"]);

    assert_eq!(code, 0);
    assert_eq!(stdout.matches("healthy config:").count(), 3);
}
