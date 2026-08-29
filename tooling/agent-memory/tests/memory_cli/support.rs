use serde_json::Value;
use std::ffi::OsStr;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
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

    pub fn root(&self) -> &Path {
        &self.root
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

#[derive(Debug, Eq, PartialEq)]
pub struct TreeSnapshot(Vec<(PathBuf, u32, u64, Option<Vec<u8>>)>);

pub fn tree_snapshot(root: &Path) -> TreeSnapshot {
    let mut entries = Vec::new();
    snapshot_entry(root, root, &mut entries);
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    TreeSnapshot(entries)
}

pub fn make_store_modes_non_private(root: &Path) {
    let metadata = fs::symlink_metadata(root).unwrap();
    let mode = if metadata.is_dir() { 0o755 } else { 0o644 };
    fs::set_permissions(root, fs::Permissions::from_mode(mode)).unwrap();
    if metadata.is_dir() {
        for entry in fs::read_dir(root).unwrap() {
            make_store_modes_non_private(&entry.unwrap().path());
        }
    }
}

pub fn only_yaml(root: &Path) -> PathBuf {
    let mut yaml = Vec::new();
    collect_yaml(root, &mut yaml);
    assert_eq!(yaml.len(), 1);
    yaml.pop().unwrap()
}

fn collect_yaml(path: &Path, yaml: &mut Vec<PathBuf>) {
    let metadata = fs::symlink_metadata(path).unwrap();
    if metadata.is_file() && path.extension() == Some(OsStr::new("yaml")) {
        yaml.push(path.to_owned());
    }
    if metadata.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            collect_yaml(&entry.unwrap().path(), yaml);
        }
    }
}

fn snapshot_entry(
    root: &Path,
    path: &Path,
    entries: &mut Vec<(PathBuf, u32, u64, Option<Vec<u8>>)>,
) {
    let metadata = fs::symlink_metadata(path).unwrap();
    let relative = path.strip_prefix(root).unwrap().to_owned();
    let bytes = metadata.is_file().then(|| fs::read(path).unwrap());
    entries.push((relative, metadata.mode() & 0o777, metadata.ino(), bytes));
    if metadata.is_dir() {
        for entry in fs::read_dir(path).unwrap() {
            snapshot_entry(root, &entry.unwrap().path(), entries);
        }
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
