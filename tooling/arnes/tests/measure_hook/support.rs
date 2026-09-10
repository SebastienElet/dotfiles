pub use serde_json::{Value, json};
use sha2::{Digest, Sha256};
pub use std::fs;
pub use std::io::{Seek, SeekFrom, Write};
pub use std::os::unix::fs::{PermissionsExt, symlink};
pub use std::path::{Path, PathBuf};
pub use std::process::{Child, Command, Output, Stdio};
pub use tempfile::TempDir;
pub struct Harness {
    pub(super) root: TempDir,
    pub(super) home: PathBuf,
    pub(super) repository: PathBuf,
    pub(super) state: PathBuf,
}
impl Harness {
    pub(super) fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Self::with_repository_name("repository")
    }
    pub(super) fn with_repository_name(
        name: &str,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let root = tempfile::tempdir()?;
        let home = root.path().join("home");
        let repository = root.path().join(name);
        let state = root.path().join("state");
        fs::create_dir(&home)?;
        fs::create_dir(&repository)?;
        fs::create_dir(&state)?;
        Ok(Self {
            root,
            home,
            repository,
            state,
        })
    }
    pub(super) fn run(
        &self,
        agent: &str,
        payload: &[u8],
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = self.command(agent).spawn()?;
        let mut stdin = child.stdin.take().ok_or("required test value is missing")?;
        match stdin.write_all(payload) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::BrokenPipe => {}
            Err(error) => return Err(format!("writing the hook payload failed: {error}").into()),
        }
        drop(stdin);
        Ok(child.wait_with_output()?)
    }
    pub(super) fn command(&self, agent: &str) -> std::process::Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_arnes"));
        command
            .args(["measure", "hook", "--agent", agent])
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        command
    }
    pub(super) fn list(&self) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Command::new(env!("CARGO_BIN_EXE_arnes"))
            .args(["measure", "list", "--format", "json"])
            .current_dir(&self.repository)
            .env_clear()
            .env("HOME", &self.home)
            .env("XDG_STATE_HOME", &self.state)
            .output()?)
    }
    pub(super) fn measure_root(&self) -> PathBuf {
        self.state.join("dotfiles/agent-harness")
    }
    pub(super) fn runs(&self) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
        let root = self.measure_root().join("runs");
        if !root.exists() {
            return Ok(Vec::new());
        }
        fs::read_dir(root)?
            .map(
                |entry| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                    Ok(entry?.path())
                },
            )
            .collect::<Result<Vec<_>, _>>()
    }
    pub(super) fn only_run(&self) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
        let runs = self.runs()?;
        assert_eq!(runs.len(), 1, "expected one run, found {runs:?}");
        Ok((*(runs).first().ok_or("missing fixture index 0")?).clone())
    }
}
pub fn assert_success(output: &Output) {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stdout.is_empty());
    assert!(
        output.stderr.is_empty(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
pub fn assert_advisory_failure(output: &Output) {
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());
}
pub fn run_at(
    harness: &Harness,
    current: &Path,
    state: &Path,
    payload: &[u8],
) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_arnes"))
        .args(["measure", "hook", "--agent", "codex"])
        .current_dir(current)
        .env_clear()
        .env("HOME", &harness.home)
        .env("XDG_STATE_HOME", state)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("required test value is missing")?
        .write_all(payload)?;
    Ok(child.wait_with_output()?)
}
pub fn run_record(
    harness: &Harness,
    session: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    read_json(
        harness
            .measure_root()
            .join("runs")
            .join(run_id("codex", session))
            .join("run.json"),
    )
}
pub fn capture_run(
    harness: &Harness,
    agent: &str,
    session_key: &str,
    session: &str,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let mut payload = json!({});
    payload
        .as_object_mut()
        .ok_or("expected hook payload object")?
        .insert(session_key.to_owned(), json!(session));
    assert_success(&harness.run(agent, payload.to_string().as_bytes())?);
    read_json(
        harness
            .measure_root()
            .join("runs")
            .join(run_id(agent, session))
            .join("run.json"),
    )
}
fn run_id(agent: &str, session: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(agent.as_bytes());
    hasher.update(session.as_bytes());
    format!("{:x}", hasher.finalize())
}
pub fn read_json(
    path: impl AsRef<Path>,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}
pub fn read_jsonl(
    path: impl AsRef<Path>,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    fs::read_to_string(path)?
        .lines()
        .map(
            |line| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                Ok(serde_json::from_str(line)?)
            },
        )
        .collect::<Result<Vec<_>, _>>()
}
pub fn walk(root: &Path) -> Result<Vec<PathBuf>, Box<dyn std::error::Error + Send + Sync>> {
    let mut paths = vec![root.to_owned()];
    let mut index = 0;
    while let Some(path) = paths.get(index).cloned() {
        index += 1;
        if path.is_dir() {
            paths.extend(
                fs::read_dir(path)?
                    .map(
                        |entry| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
                            Ok(entry?.path())
                        },
                    )
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
    }
    Ok(paths)
}
pub fn init_repository(
    repository: &Path,
    branch: &str,
    tracked: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    git(repository, &["init", "-b", branch])?;
    fs::write(repository.join(tracked), tracked)?;
    git(repository, &["add", tracked])?;
    commit(repository, "initial")?;
    Ok(())
}
pub fn git(
    repository: &Path,
    args: &[&str],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()?;
    assert!(
        output.status.success(),
        "git {args:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(())
}
pub fn commit(
    repository: &Path,
    message: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    git(
        repository,
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-m",
            message,
        ],
    )?;
    Ok(())
}
pub fn git_value(
    repository: &Path,
    args: &[&str],
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let output = Command::new("git")
        .args(args)
        .current_dir(repository)
        .output()?;
    assert!(output.status.success());
    Ok(String::from_utf8(output.stdout)?.trim().to_owned())
}
