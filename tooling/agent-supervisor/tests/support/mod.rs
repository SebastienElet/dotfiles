use serde_json::{Value, json};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn command(arguments: &[&str]) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_agent-supervisor"));
    command
        .args(arguments)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

pub fn invoke(arguments: &[&str], input: Option<&Value>) -> Result<Output> {
    let mut child = command(arguments).spawn()?;
    if let Some(input) = input {
        child
            .stdin
            .take()
            .ok_or("stdin unavailable")?
            .write_all(&serde_json::to_vec(input)?)?;
    }
    Ok(child.wait_with_output()?)
}

pub struct Fixture {
    pub root: tempfile::TempDir,
    pub repository: PathBuf,
    pub directory: String,
    pub manifest: Value,
}

impl Fixture {
    pub fn new() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let repository = root.path().join("repository");
        fs::create_dir(&repository)?;
        assert!(
            Command::new("git")
                .args(["init", "--quiet"])
                .arg(&repository)
                .status()?
                .success()
        );
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&repository)
                .args([
                    "-c",
                    "user.name=Synthetic",
                    "-c",
                    "user.email=test@example.invalid",
                    "-c",
                    "core.hooksPath=/dev/null",
                    "commit",
                    "--allow-empty",
                    "--no-gpg-sign",
                    "-qm",
                    "fixture"
                ])
                .status()?
                .success()
        );
        let directory = root
            .path()
            .join("private-task")
            .to_str()
            .ok_or("non UTF-8 fixture")?
            .to_owned();
        let manifest = json!({
            "task_id":"synthetic", "owner":"test-owner", "repository":repository,
            "goal":"finish the synthetic task", "permissions":["edit this repository"],
            "codex":env!("CARGO_BIN_EXE_supervisor-test-agent"),
            "codexbar":env!("CARGO_BIN_EXE_supervisor-test-agent"),
            "handoff":env!("CARGO_BIN_EXE_supervisor-test-agent"),
            "model":"test-model", "sandbox":"read-only",
            "expires_at":SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()+60,
            "max_turns":2, "max_capacity_probes":4, "retry_seconds":1, "turn_timeout_seconds":10
        });
        Ok(Self {
            root,
            repository,
            directory,
            manifest,
        })
    }

    pub fn init(&self) -> Result<()> {
        let result = invoke(&["init", &self.directory], Some(&self.manifest))?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        fs::write(
            std::path::Path::new(&self.directory).join("fixture-skill.md"),
            "Synthetic handoff skill\n",
        )?;
        Ok(())
    }

    pub fn run(&self) -> Result<Value> {
        let result = invoke(&["run", &self.directory], None)?;
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        Ok(serde_json::from_slice(&result.stdout)?)
    }

    pub fn count(&self) -> Result<String> {
        Ok(fs::read_to_string(self.repository.join("count"))?)
    }

    pub fn scenario(&self, name: &str) -> Result<()> {
        Ok(fs::write(self.repository.join("scenario"), name)?)
    }
}
