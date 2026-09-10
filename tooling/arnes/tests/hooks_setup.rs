#![cfg(test)]
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
    fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let home = root.path().join("home");
        let repository = root.path().join("repository");
        fs::create_dir(&home)?;
        fs::create_dir(&repository)?;
        let repository = fs::canonicalize(repository)?;
        Ok(Self {
            _root: root,
            home,
            repository,
        })
    }
    fn write_manifest(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_manifest_contents(
            r"version: 1
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
",
        )?;
        Ok(())
    }
    fn write_handoff_only_manifest(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_manifest_contents(
            r"version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: handoff
    installations:
      - { agent: claude, scope: user }
resources: []
",
        )?;
        Ok(())
    }
    fn write_measurement_only_manifest(
        &self,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_manifest_contents(
            r"version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: measurement
    installations:
      - { agent: claude, scope: user }
resources: []
",
        )?;
        Ok(())
    }
    fn write_codex_handoff_manifest(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_manifest_contents(
            r"version: 1
agents:
  - id: codex
    scopes: [user]
hooks:
  - id: handoff
    installations:
      - { agent: codex, scope: user }
resources: []
",
        )?;
        Ok(())
    }
    fn write_memory_manifest(
        &self,
        agent: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self . write_manifest_contents (& format ! ("version: 1\nagents:\n  - id: {agent}\n    scopes: [user]\nhooks:\n  - id: memory\n    installations:\n      - {{ agent: {agent}, scope: user }}\nresources: []\n")) ? ;
        Ok(())
    }
    fn write_invalid_manifest(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.write_manifest_contents(
            r"version: 1
agents:
  - id: claude
    scopes: [user]
hooks:
  - id: unknown
    installations:
      - { agent: claude, scope: user }
resources: []
",
        )?;
        Ok(())
    }
    fn write_manifest_contents(
        &self,
        contents: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        fs::write(self.home.join(".arnes.yaml"), contents)?;
        Ok(())
    }
    fn executable(&self, name: &str) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let path = self.home.join(".local/bin").join(name);
        fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
        fs::write(&path, b"binary")?;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
        Ok(path)
    }
    fn setup(&self, agent: &str) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["setup", "hooks", "--agent", agent])
            .env_clear()
            .env("HOME", &self.home)
            .current_dir(&self.repository)
            .output()?)
    }
    fn config(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        read_json(&self.home.join(".claude/settings.json"))
    }
    fn codex_config(&self) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        read_json(&self.home.join(".codex/hooks.json"))
    }
    fn write_claude_config(
        &self,
        value: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = self.home.join(".claude/settings.json");
        fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
        fs::write(path, serde_json::to_vec(value)?)?;
        Ok(())
    }
    fn write_codex_config(
        &self,
        value: &Value,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let path = self.home.join(".codex/hooks.json");
        fs::create_dir_all(path.parent().ok_or("fixture path has no parent")?)?;
        fs::write(path, serde_json::to_vec(value)?)?;
        Ok(())
    }
}
fn read_json(path: &Path) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
#[test]
fn setup_installs_manifest_hooks_in_one_agent_configuration()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_manifest()?;
    let arnes = harness.executable("arnes")?;
    let handoff = harness.executable("agent-handoff")?;
    let output = harness.setup("claude")?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let config = harness.config()?;
    let commands: Vec<&str> = (*(config).get("hooks").ok_or("missing fixture index hooks")?)
        .as_object()
        .ok_or("expected JSON object")?
        .values()
        .map(
            |entries| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(entries.as_array().ok_or("expected JSON array")?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .map(
            |group| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(
                    (*(group).get("hooks").ok_or("missing fixture index hooks")?)
                        .as_array()
                        .ok_or("expected JSON array")?,
                )
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .map(
            |handler| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok((*(handler)
                    .get("command")
                    .ok_or("missing fixture index command")?)
                .as_str()
                .ok_or("expected JSON string")?)
            },
        )
        .collect::<Result<Vec<_>, _>>()?;
    assert!(
        commands
            .contains(&format!("'{}' measure hook --agent claude-code", arnes.display()).as_str())
    );
    assert!(commands.contains(&handoff.to_str().ok_or("expected JSON string")?));
    Ok(())
}
#[test]
fn setup_installs_memory_prompt_hook_for_codex_and_claude()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for agent in ["codex", "claude"] {
        let harness = Harness::new()?;
        harness.write_memory_manifest(agent)?;
        let memory = harness.executable("agent-memory")?;
        let output = harness.setup(agent)?;
        assert_eq!(
            output.status.code(),
            Some(0),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let config = if agent == "codex" {
            harness.codex_config()?
        } else {
            harness.config()?
        };
        let hooks = (*(config).get("hooks").ok_or("missing fixture index hooks")?)
            .as_object()
            .ok_or("expected JSON object")?;
        assert_eq!(hooks.len(), 1);
        let handler = (*(*(*(hooks)
            .get("UserPromptSubmit")
            .ok_or("missing fixture index UserPromptSubmit")?)
        .get(0)
        .ok_or("missing fixture index 0")?)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
        .get(0)
        .ok_or("missing fixture index 0")?;
        assert_eq!(
            *(handler).get("type").ok_or("missing fixture index type")?,
            "command"
        );
        assert_eq!(
            *(handler)
                .get("command")
                .ok_or("missing fixture index command")?,
            format!("'{}' hook --agent {agent}", memory.display())
        );
        assert_eq!(
            *(handler)
                .get("timeout")
                .ok_or("missing fixture index timeout")?,
            30
        );
    };
    Ok(())
}
#[test]
fn setup_rejects_memory_for_cursor_without_creating_configuration()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_memory_manifest("cursor")?;
    harness.executable("agent-memory")?;
    let output = harness.setup("cursor")?;
    assert_eq!(output.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("Cursor does not support the memory hook")
    );
    assert!(!harness.home.join(".cursor/hooks.json").exists());
    Ok(())
}
#[test]
fn setup_rejects_invalid_memory_runtime_without_changing_configuration()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for case in ["missing", "directory", "non-executable"] {
        let harness = Harness::new()?;
        harness.write_memory_manifest("codex")?;
        let initial = serde_json :: json ! ({ "hooks" : { "UserPromptSubmit" : [{ "hooks" : [{ "type" : "command" , "command" : "third-party" }] }] } });
        harness.write_codex_config(&initial)?;
        let memory = harness.home.join(".local/bin/agent-memory");
        match case {
            "missing" => {}
            "directory" => fs::create_dir_all(&memory)?,
            "non-executable" => {
                fs::create_dir_all(memory.parent().ok_or("fixture path has no parent")?)?;
                fs::write(&memory, b"binary")?;
                fs::set_permissions(&memory, fs::Permissions::from_mode(0o600))?;
            }
            _ => return Err("unsupported hook fixture shape".to_string().into()),
        }
        let output = harness.setup("codex")?;
        assert_eq!(output.status.code(), Some(2), "{case}");
        assert_eq!(harness.codex_config()?, initial, "{case}");
    };
    Ok(())
}
#[test]
fn setup_installs_handoff_without_requiring_measurement()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_handoff_only_manifest()?;
    let handoff = harness.executable("agent-handoff")?;
    let old_source = harness.repository.join("tooling/agent-handoff");
    let old_script = harness.repository.join("scripts/agent_handoff");
    let third_party = serde_json :: json ! ({ "type" : "command" , "command" : "third-party" , "args" : ["keep"] , "async" : true });
    harness . write_claude_config (& serde_json :: json ! ({ "hooks" : { "Stop" : [{ "hooks" : [{ "type" : "command" , "command" : old_source , "timeout" : 7 } , third_party , { "type" : "command" , "command" : old_script }] }] } })) ? ;
    let output = harness.setup("claude")?;
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let config = harness.config()?;
    let serialized = serde_json::to_string(&config)?;
    assert!(
        !serialized.contains(
            old_source
                .to_str()
                .ok_or("required test value is missing")?
        )
    );
    assert!(
        !serialized.contains(
            old_script
                .to_str()
                .ok_or("required test value is missing")?
        )
    );
    let handlers: Vec<&Value> = (*(*(config).get("hooks").ok_or("missing fixture index hooks")?)
        .get("Stop")
        .ok_or("missing fixture index Stop")?)
    .as_array()
    .ok_or("expected JSON array")?
    .iter()
    .map(
        |group| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
            Ok(
                (*(group).get("hooks").ok_or("missing fixture index hooks")?)
                    .as_array()
                    .ok_or("expected JSON array")?,
            )
        },
    )
    .collect::<Result<Vec<_>, _>>()?
    .into_iter()
    .flatten()
    .collect();
    assert!(handlers.contains(&&third_party));
    let handoff_command = handoff
        .to_str()
        .ok_or("fixture command path is not UTF-8")?;
    let current = handlers
        .iter()
        .map(
            |handler| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                let include = {
                    (*(handler)
                        .get("command")
                        .ok_or("missing fixture index command")?)
                    .as_str()
                        == Some(handoff_command)
                };
                Ok((include, handler))
            },
        )
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .find(|(include, _)| *include)
        .map(|(_, entry)| entry)
        .ok_or("required test value is missing")?;
    assert_eq!(
        *(current).get("args").ok_or("missing fixture index args")?,
        serde_json::json!([])
    );
    assert_eq!(
        *(current)
            .get("timeout")
            .ok_or("missing fixture index timeout")?,
        7
    );
    Ok(())
}
#[test]
fn setup_rejects_an_undeclared_agent_without_creating_configuration()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_handoff_only_manifest()?;
    let output = harness.setup("cursor")?;
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("is not declared for user scope"));
    assert!(!harness.home.join(".cursor/hooks.json").exists());
    Ok(())
}
#[test]
fn setup_rejects_an_invalid_manifest_without_changing_configuration()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_invalid_manifest()?;
    let initial = serde_json :: json ! ({ "hooks" : { "Stop" : [{ "hooks" : [{ "type" : "command" , "command" : "third-party" }] }] } });
    harness.write_claude_config(&initial)?;
    let output = harness.setup("claude")?;
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown variant"));
    assert_eq!(harness.config()?, initial);
    Ok(())
}
#[test]
fn setup_removes_owned_capabilities_absent_from_the_manifest()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_measurement_only_manifest()?;
    let arnes = harness.executable("arnes")?;
    let handoff = harness.home.join(".local/bin/agent-handoff");
    let old_source = harness.repository.join("tooling/agent-handoff");
    let old_script = harness.repository.join("scripts/agent_handoff");
    harness . write_claude_config (& serde_json :: json ! ({ "hooks" : { "Stop" : [{ "hooks" : [{ "type" : "command" , "command" : handoff } , { "type" : "command" , "command" : old_source } , { "type" : "command" , "command" : old_script } , { "type" : "command" , "command" : "third-party" }] }] } })) ? ;
    let output = harness.setup("claude")?;
    assert_eq!(output.status.code(), Some(0));
    let serialized = serde_json::to_string(&harness.config()?)?;
    assert!(!serialized.contains(handoff.to_str().ok_or("required test value is missing")?));
    assert!(
        !serialized.contains(
            old_source
                .to_str()
                .ok_or("required test value is missing")?
        )
    );
    assert!(
        !serialized.contains(
            old_script
                .to_str()
                .ok_or("required test value is missing")?
        )
    );
    assert!(serialized.contains("third-party"));
    assert!(serialized.contains(&format!(
        "'{}' measure hook --agent claude-code",
        arnes.display()
    )));
    Ok(())
}
#[test]
fn setup_uses_the_codex_handoff_shape() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let harness = Harness::new()?;
    harness.write_codex_handoff_manifest()?;
    let handoff = harness.executable("agent-handoff")?;
    let output = harness.setup("codex")?;
    assert_eq!(output.status.code(), Some(0));
    let codex_configuration = harness.codex_config()?;
    let handler = (*(*(*(*(codex_configuration)
        .get("hooks")
        .ok_or("missing fixture index hooks")?)
    .get("Stop")
    .ok_or("missing fixture index Stop")?)
    .get(0)
    .ok_or("missing fixture index 0")?)
    .get("hooks")
    .ok_or("missing fixture index hooks")?)
    .get(0)
    .ok_or("missing fixture index 0")?;
    assert_eq!(
        *(handler)
            .get("command")
            .ok_or("missing fixture index command")?,
        handoff.to_str().ok_or("required test value is missing")?
    );
    assert!(handler.get("args").is_none());
    Ok(())
}
