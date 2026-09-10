use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
pub(super) struct CliFixture {
    _temporary: tempfile::TempDir,
    repository: PathBuf,
    root: PathBuf,
}
impl CliFixture {
    pub(super) fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let temporary = tempfile::tempdir()?;
        let repository = temporary.path().join("repository");
        fs::create_dir(&repository)?;
        git(&repository, ["init", "-q"])?;
        fs::write(repository.join("proof.txt"), b"durable proof")?;
        git(&repository, ["add", "proof.txt"])?;
        let root = temporary.path().join("store");
        Ok(Self {
            _temporary: temporary,
            repository,
            root,
        })
    }
    pub(super) fn git_draft(
        kind: &str,
        statement: &str,
        retrieval_term: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Ok (format ! ("schema_version: 1\nkind: {kind}\nstatement: {}\nretrieval_terms:\n  - {}\nproof:\n  summary: The tracked proof establishes this memory.\n  sources:\n    - kind: git-file\n      locator: proof.txt\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the tracked proof remain valid?\n    valid_when: The tracked file remains authoritative.\n  outcomes:\n    valid: The memory remains valid.\n    invalidated: The tracked proof changed.\n" , serde_json :: to_string (statement) ? , serde_json :: to_string (retrieval_term) ?) . into_bytes ())
    }
    pub(super) fn repository(&self) -> &Path {
        &self.repository
    }
    pub(super) fn root(&self) -> &Path {
        &self.root
    }
    pub(super) fn run<const N: usize>(
        &self,
        arguments: [&str; N],
        stdin: &[u8],
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
            .args(arguments)
            .current_dir(&self.repository)
            .env("AGENT_MEMORY_ROOT", &self.root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        std::io::Write::write_all(
            child.stdin.as_mut().ok_or("child stdin is not piped")?,
            stdin,
        )?;
        Ok(child.wait_with_output()?)
    }
}
pub(super) fn assert_exit(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}
pub(super) fn assert_error(
    output: &Output,
    exit: i32,
    code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_exit(output, exit);
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(
        (*error
            .pointer("/error/code")
            .ok_or("missing hook response /error/code")?),
        code
    );
    assert!(output.stdout.is_empty());
    Ok(())
}
pub(super) fn stdout_json(
    output: &Output,
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    Ok(serde_json::from_slice(&output.stdout)?)
}
fn git<const N: usize>(
    directory: &Path,
    arguments: [&str; N],
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .status()?
            .success()
    );
    Ok(())
}
