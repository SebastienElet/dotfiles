use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn moon_home() -> PathBuf {
    std::env::var_os("MOON_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("XDG_DATA_HOME").map(|path| PathBuf::from(path).join("moon")))
        .or_else(|| std::env::var_os("HOME").map(|path| PathBuf::from(path).join(".moon")))
        .expect("Moon tests require HOME, XDG_DATA_HOME, or MOON_HOME")
}

fn proto_home() -> PathBuf {
    std::env::var_os("PROTO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|path| PathBuf::from(path).join(".proto")))
        .expect("Moon tests require HOME or PROTO_HOME")
}

fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

#[test]
fn moon_deployments_satisfy_instruction_rule_and_skill_doctors() {
    let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let home = tempfile::tempdir().unwrap();
    let deployment =
        Command::new(std::env::var_os("DEPLOYMENT_MOON").unwrap_or_else(|| "moon".into()))
            .args([
                "exec",
                "--quiet",
                "--ignore-ci-checks",
                "--no-actions",
                "--upstream",
                "none",
                "home:arnes-config",
                "harness:codex-instructions",
                "harness:claude-rules",
                "harness:codex-skills",
            ])
            .env("HOME", home.path())
            .env("MOON_HOME", moon_home())
            .env("PROTO_HOME", proto_home())
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
