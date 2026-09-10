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
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
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
    pub fn git_draft(
        kind: &str,
        statement: &str,
        retrieval_term: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        Ok (format ! ("schema_version: 1\nkind: {kind}\nstatement: {}\nretrieval_terms:\n  - {}\nproof:\n  summary: The tracked proof establishes this memory.\n  sources:\n    - kind: git-file\n      locator: proof.txt\noracle:\n  automated:\n    kind: source-fingerprint\n    expected: all-proof-sources-unchanged\n  human_fallback:\n    question: Does the tracked proof remain valid?\n    valid_when: The tracked file remains authoritative.\n  outcomes:\n    valid: The memory remains valid.\n    invalidated: The tracked proof changed.\n" , serde_json :: to_string (statement) ? , serde_json :: to_string (retrieval_term) ?) . into_bytes ())
    }
    pub fn user_git_draft(
        statement: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
        let draft = String::from_utf8(Self::git_draft("invariant", statement, "user git proof")?)?;
        Ok(draft
            .replacen("statement:", "scope: user\nstatement:", 1)
            .into_bytes())
    }
    pub fn repository(&self) -> &Path {
        &self.repository
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn run<const N: usize>(
        &self,
        arguments: [&str; N],
        stdin: &[u8],
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        self.run_with_root(arguments, stdin, &self.root)
    }
    pub fn run_with_root<const N: usize>(
        &self,
        arguments: [&str; N],
        stdin: &[u8],
        root: impl AsRef<OsStr>,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
            .args(arguments)
            .current_dir(&self.repository)
            .env("AGENT_MEMORY_ROOT", root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        std::io::Write::write_all(
            child.stdin.as_mut().ok_or("child stdin is not piped")?,
            stdin,
        )?;
        Ok(child.wait_with_output()?)
    }
    pub fn run_from_with_root<const N: usize>(
        directory: &Path,
        arguments: [&str; N],
        stdin: &[u8],
        root: impl AsRef<OsStr>,
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
            .args(arguments)
            .current_dir(directory)
            .env("AGENT_MEMORY_ROOT", root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        std::io::Write::write_all(
            child.stdin.as_mut().ok_or("child stdin is not piped")?,
            stdin,
        )?;
        Ok(child.wait_with_output()?)
    }
    pub fn run_after_cwd_removal<const N: usize>(
        &self,
        directory: &Path,
        arguments: [&str; N],
        stdin: &[u8],
    ) -> Result<Output, Box<dyn std::error::Error + Send + Sync>> {
        let mut child = Command::new(env!("CARGO_BIN_EXE_agent-memory"))
            .args(arguments)
            .current_dir(directory)
            .env("AGENT_MEMORY_ROOT", &self.root)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()?;
        fs::remove_dir(directory)?;
        std::io::Write::write_all(
            child.stdin.as_mut().ok_or("child stdin is not piped")?,
            stdin,
        )?;
        Ok(child.wait_with_output()?)
    }
}
#[derive(Debug, Eq, PartialEq)]
pub struct TreeSnapshot(Vec<(PathBuf, u32, u64, Option<Vec<u8>>)>);
pub fn tree_snapshot(
    root: &Path,
) -> Result<TreeSnapshot, Box<dyn std::error::Error + Send + Sync>> {
    let mut entries = Vec::new();
    snapshot_entry(root, root, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(TreeSnapshot(entries))
}
pub fn make_store_modes_non_private(
    root: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let metadata = fs::symlink_metadata(root)?;
    let mode = if metadata.is_dir() { 0o755 } else { 0o644 };
    fs::set_permissions(root, fs::Permissions::from_mode(mode))?;
    let _: () = if metadata.is_dir() {
        for entry in fs::read_dir(root)? {
            make_store_modes_non_private(&entry?.path())?;
        }
    };
    Ok(())
}
pub fn only_yaml(root: &Path) -> Result<PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    let mut yaml = Vec::new();
    collect_yaml(root, &mut yaml)?;
    assert_eq!(yaml.len(), 1);
    Ok(yaml.pop().ok_or("expected fixture item")?)
}
fn collect_yaml(
    path: &Path,
    yaml: &mut Vec<PathBuf>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.is_file() && path.extension() == Some(OsStr::new("yaml")) {
        yaml.push(path.to_owned());
    }
    let _: () = if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            collect_yaml(&entry?.path(), yaml)?;
        }
    };
    Ok(())
}
fn snapshot_entry(
    root: &Path,
    path: &Path,
    entries: &mut Vec<(PathBuf, u32, u64, Option<Vec<u8>>)>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let metadata = fs::symlink_metadata(path)?;
    let relative = path.strip_prefix(root)?.to_owned();
    let bytes = metadata.is_file().then(|| fs::read(path)).transpose()?;
    entries.push((relative, metadata.mode() & 0o777, metadata.ino(), bytes));
    let _: () = if metadata.is_dir() {
        for entry in fs::read_dir(path)? {
            snapshot_entry(root, &entry?.path(), entries)?;
        }
    };
    Ok(())
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
pub fn assert_error(
    output: &Output,
    exit: i32,
    code: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_exit(output, exit);
    let error: Value = serde_json::from_slice(&output.stderr)?;
    assert_eq!(
        (*(*(error)
            .get("error")
            .ok_or(concat!("missing fixture index ", stringify!("error")))?)
        .get("code")
        .ok_or(concat!("missing fixture index ", stringify!("code")))?),
        code
    );
    assert!(output.stdout.is_empty());
    Ok(())
}
pub fn stdout_json(output: &Output) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
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
