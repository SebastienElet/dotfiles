use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

pub struct CliFixture {
    _temporary: tempfile::TempDir,
    repository: PathBuf,
    root: PathBuf,
}

impl CliFixture {
    pub fn new() -> Self {
        let temporary = tempfile::tempdir().unwrap();
        let repository = temporary.path().join("repository");
        fs::create_dir(&repository).unwrap();
        git(&repository, ["init", "-q"]);
        fs::write(repository.join("proof.txt"), b"durable proof").unwrap();
        git(&repository, ["add", "proof.txt"]);
        let root = temporary.path().join("store");
        Self {
            _temporary: temporary,
            repository,
            root,
        }
    }

    pub fn git_draft(&self, kind: &str, statement: &str, retrieval_term: &str) -> Vec<u8> {
        format!(
            "schema_version: 1\nkind: {kind}\nstatement: {}\nretrieval_terms:\n  - {}\nproof:\n  summary: The tracked proof establishes this memory.\n  sources:\n    - kind: git-file\n      locator: proof.txt\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the tracked proof remain valid?\n    valid_when: The tracked file remains authoritative.\n  outcomes:\n    valid: The memory remains valid.\n    invalidated: The tracked proof changed.\n",
            serde_json::to_string(statement).unwrap(),
            serde_json::to_string(retrieval_term).unwrap(),
        )
        .into_bytes()
    }

    pub fn run<const N: usize>(&self, arguments: [&str; N], stdin: &[u8]) -> Output {
        self.run_with_root(arguments, stdin, &self.root)
    }

    pub fn run_with_root<const N: usize>(
        &self,
        arguments: [&str; N],
        stdin: &[u8],
        root: impl AsRef<OsStr>,
    ) -> Output {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
            .args(arguments)
            .current_dir(&self.repository)
            .env("AGENT_MEMORY_ROOT", root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        std::io::Write::write_all(child.stdin.as_mut().unwrap(), stdin).unwrap();
        child.wait_with_output().unwrap()
    }
}

pub fn assert_exit(output: &Output, code: i32) {
    assert_eq!(
        output.status.code(),
        Some(code),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

pub fn assert_error(output: &Output, exit: i32, code: &str) {
    assert_exit(output, exit);
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], code);
    assert!(output.stdout.is_empty());
}

pub fn stdout_json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

fn git<const N: usize>(directory: &Path, arguments: [&str; N]) {
    assert!(
        Command::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .status()
            .unwrap()
            .success()
    );
}
