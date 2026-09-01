use serde_json::Value;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use tempfile::TempDir;

struct Harness {
    _root: TempDir,
    home: PathBuf,
    repository: PathBuf,
}

impl Harness {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let home = root.path().join("home");
        let repository = root.path().join("repository");
        fs::create_dir(&home).unwrap();
        fs::create_dir(&repository).unwrap();
        let repository = fs::canonicalize(repository).unwrap();
        Self {
            _root: root,
            home,
            repository,
        }
    }

    fn write_manifest(&self) {
        self.write_manifest_contents(
            r#"version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: measurement
    installations:
      - { agent: claude, scope: user }
  - id: handoff
    installations:
      - { agent: claude, scope: user }
resources: []
"#,
        );
    }

    fn write_handoff_only_manifest(&self) {
        self.write_manifest_contents(
            r#"version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: handoff
    installations:
      - { agent: claude, scope: user }
resources: []
"#,
        );
    }

    fn write_measurement_only_manifest(&self) {
        self.write_manifest_contents(
            r#"version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: measurement
    installations:
      - { agent: claude, scope: user }
resources: []
"#,
        );
    }

    fn write_codex_handoff_manifest(&self) {
        self.write_manifest_contents(
            r#"version: 1
agents:
  - id: codex
    scopes: [user]
hooks:
  - id: handoff
    installations:
      - { agent: codex, scope: user }
resources: []
"#,
        );
    }

    fn write_memory_manifest(&self, agent: &str) {
        self.write_manifest_contents(&format!(
            "version: 1\nagents:\n  - id: {agent}\n    scopes: [user]\nhooks:\n  - id: memory\n    installations:\n      - {{ agent: {agent}, scope: user }}\nresources: []\n"
        ));
    }

    fn write_invalid_manifest(&self) {
        self.write_manifest_contents(
            r#"version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: unknown
    installations:
      - { agent: claude, scope: user }
resources: []
"#,
        );
    }

    fn write_manifest_contents(&self, contents: &str) {
        fs::write(self.home.join(".arnes.yaml"), contents).unwrap();
    }

    fn executable(&self, name: &str) -> PathBuf {
        let path = self.home.join(".local/bin").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, b"binary").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
        path
    }

    fn setup(&self, agent: &str) -> Output {
        Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["setup", "hooks", "--agent", agent])
            .env_clear()
            .env("HOME", &self.home)
            .current_dir(&self.repository)
            .output()
            .unwrap()
    }

    fn config(&self) -> Value {
        read_json(&self.home.join(".claude/settings.json"))
    }

    fn codex_config(&self) -> Value {
        read_json(&self.home.join(".codex/hooks.json"))
    }

    fn write_claude_config(&self, value: &Value) {
        let path = self.home.join(".claude/settings.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    }

    fn write_codex_config(&self, value: &Value) {
        let path = self.home.join(".codex/hooks.json");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    }
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
fn setup_installs_manifest_hooks_in_one_agent_configuration() {
    let harness = Harness::new();
    harness.write_manifest();
    let arnes = harness.executable("arnes");
    let handoff = harness.executable("agent-handoff");
    let output = harness.setup("claude");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let config = harness.config();
    let commands: Vec<&str> = config["hooks"]
        .as_object()
        .unwrap()
        .values()
        .flat_map(|entries| entries.as_array().unwrap())
        .flat_map(|group| group["hooks"].as_array().unwrap())
        .map(|handler| handler["command"].as_str().unwrap())
        .collect();
    assert!(
        commands
            .contains(&format!("'{}' measure hook --agent claude-code", arnes.display()).as_str())
    );
    assert!(commands.contains(&handoff.to_str().unwrap()));
}

#[test]
fn setup_installs_memory_prompt_hook_for_codex_and_claude() {
    for agent in ["codex", "claude"] {
        let harness = Harness::new();
        harness.write_memory_manifest(agent);
        let memory = harness.executable("agent-memory");

        let output = harness.setup(agent);

        assert_eq!(
            output.status.code(),
            Some(0),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let config = if agent == "codex" {
            harness.codex_config()
        } else {
            harness.config()
        };
        let hooks = config["hooks"].as_object().unwrap();
        assert_eq!(hooks.len(), 1);
        let handler = &hooks["UserPromptSubmit"][0]["hooks"][0];
        assert_eq!(handler["type"], "command");
        assert_eq!(
            handler["command"],
            format!("'{}' hook --agent {agent}", memory.display())
        );
        assert_eq!(handler["timeout"], 30);
    }
}

#[test]
fn setup_rejects_memory_for_cursor_without_creating_configuration() {
    let harness = Harness::new();
    harness.write_memory_manifest("cursor");
    harness.executable("agent-memory");

    let output = harness.setup("cursor");

    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Cursor does not support the memory hook")
    );
    assert!(!harness.home.join(".cursor/hooks.json").exists());
}

#[test]
fn setup_rejects_invalid_memory_runtime_without_changing_configuration() {
    for case in ["missing", "directory", "non-executable"] {
        let harness = Harness::new();
        harness.write_memory_manifest("codex");
        let initial = serde_json::json!({"hooks":{"UserPromptSubmit":[{"hooks":[
            {"type":"command","command":"third-party"}
        ]}]}});
        harness.write_codex_config(&initial);
        let memory = harness.home.join(".local/bin/agent-memory");
        match case {
            "missing" => {}
            "directory" => fs::create_dir_all(&memory).unwrap(),
            "non-executable" => {
                fs::create_dir_all(memory.parent().unwrap()).unwrap();
                fs::write(&memory, b"binary").unwrap();
                fs::set_permissions(&memory, fs::Permissions::from_mode(0o600)).unwrap();
            }
            _ => unreachable!(),
        }

        let output = harness.setup("codex");

        assert_eq!(output.status.code(), Some(2), "{case}");
        assert_eq!(harness.codex_config(), initial, "{case}");
    }
}

#[test]
fn setup_installs_handoff_without_requiring_measurement() {
    let harness = Harness::new();
    harness.write_handoff_only_manifest();
    let handoff = harness.executable("agent-handoff");
    let old_source = harness.repository.join("tooling/agent-handoff");
    let old_script = harness.repository.join("scripts/agent_handoff");
    let third_party = serde_json::json!({
        "type":"command", "command":"third-party", "args":["keep"], "async":true
    });
    harness.write_claude_config(&serde_json::json!({"hooks":{"Stop":[{"hooks":[
        {"type":"command","command":old_source,"timeout":7},
        third_party,
        {"type":"command","command":old_script}
    ]}]}}));
    let output = harness.setup("claude");
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let config = harness.config();
    let serialized = serde_json::to_string(&config).unwrap();
    assert!(!serialized.contains(old_source.to_str().unwrap()));
    assert!(!serialized.contains(old_script.to_str().unwrap()));
    let handlers: Vec<&Value> = config["hooks"]["Stop"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|group| group["hooks"].as_array().unwrap())
        .collect();
    assert!(handlers.contains(&&third_party));
    let current = handlers
        .iter()
        .find(|handler| handler["command"] == handoff.to_str().unwrap())
        .unwrap();
    assert_eq!(current["args"], serde_json::json!([]));
    assert_eq!(current["timeout"], 7);
}

#[test]
fn setup_rejects_an_undeclared_agent_without_creating_configuration() {
    let harness = Harness::new();
    harness.write_handoff_only_manifest();

    let output = harness.setup("cursor");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not declared for user scope"));
    assert!(!harness.home.join(".cursor/hooks.json").exists());
}

#[test]
fn setup_rejects_an_invalid_manifest_without_changing_configuration() {
    let harness = Harness::new();
    harness.write_invalid_manifest();
    let initial = serde_json::json!({"hooks":{"Stop":[{"hooks":[
        {"type":"command","command":"third-party"}
    ]}]}});
    harness.write_claude_config(&initial);

    let output = harness.setup("claude");

    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown variant"));
    assert_eq!(harness.config(), initial);
}

#[test]
fn setup_removes_owned_capabilities_absent_from_the_manifest() {
    let harness = Harness::new();
    harness.write_measurement_only_manifest();
    let arnes = harness.executable("arnes");
    let handoff = harness.home.join(".local/bin/agent-handoff");
    let old_source = harness.repository.join("tooling/agent-handoff");
    let old_script = harness.repository.join("scripts/agent_handoff");
    harness.write_claude_config(&serde_json::json!({"hooks":{"Stop":[{"hooks":[
        {"type":"command","command":handoff},
        {"type":"command","command":old_source},
        {"type":"command","command":old_script},
        {"type":"command","command":"third-party"}
    ]}]}}));

    let output = harness.setup("claude");

    assert_eq!(output.status.code(), Some(0));
    let serialized = serde_json::to_string(&harness.config()).unwrap();
    assert!(!serialized.contains(handoff.to_str().unwrap()));
    assert!(!serialized.contains(old_source.to_str().unwrap()));
    assert!(!serialized.contains(old_script.to_str().unwrap()));
    assert!(serialized.contains("third-party"));
    assert!(serialized.contains(&format!(
        "'{}' measure hook --agent claude-code",
        arnes.display()
    )));
}

#[test]
fn setup_uses_the_codex_handoff_shape() {
    let harness = Harness::new();
    harness.write_codex_handoff_manifest();
    let handoff = harness.executable("agent-handoff");

    let output = harness.setup("codex");

    assert_eq!(output.status.code(), Some(0));
    let handler = &harness.codex_config()["hooks"]["Stop"][0]["hooks"][0];
    assert_eq!(handler["command"], handoff.to_str().unwrap());
    assert!(handler.get("args").is_none());
}
