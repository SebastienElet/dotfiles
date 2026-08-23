use std::path::Path;
use std::process::{Command, Output};

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn make_deployments_satisfy_instruction_rule_and_skill_doctors() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let home = tempfile::tempdir().unwrap();
    let codex_instructions = home.path().join(".codex/AGENTS.md");
    let claude_rule = home.path().join(".claude/rules/agent-instructions.md");
    let codex_skill = home.path().join(".agents/skills/agent-instructions");
    let manifest = home.path().join(".arnes.yaml");
    let deployment = Command::new("make")
        .arg("-f")
        .arg(repository.join("Makefile"))
        .arg(format!("DOTFILES_PATH={}", repository.display()))
        .arg(&manifest)
        .arg(&codex_instructions)
        .arg(&claude_rule)
        .arg(&codex_skill)
        .env("HOME", home.path())
        .current_dir(&repository)
        .output()
        .unwrap();
    assert!(deployment.status.success(), "{}", output_text(&deployment));

    for (resource, agent) in [("instructions", "codex"), ("rules", "claude")] {
        let diagnosis = Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["doctor", resource, "--agent", agent, "--scope", "user"])
            .env("HOME", home.path())
            .current_dir(&repository)
            .output()
            .unwrap();
        assert!(diagnosis.status.success(), "{}", output_text(&diagnosis));
    }

    let skill_diagnosis = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args([
            "doctor", "skills", "--agent", "codex", "--scope", "user", "--format", "json",
        ])
        .env("HOME", home.path())
        .current_dir(&repository)
        .output()
        .unwrap();
    let diagnostics: serde_json::Value = serde_json::from_slice(&skill_diagnosis.stdout).unwrap();
    assert!(diagnostics.as_array().unwrap().iter().any(|diagnostic| {
        diagnostic["resource"] == "skills"
            && diagnostic["state"] == "healthy"
            && diagnostic["message"]
                .as_str()
                .is_some_and(|message| message.contains("agent-instructions"))
    }));
}
