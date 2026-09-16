use crate::{
    Result,
    classify::classify,
    content,
    model::{Epoch, State, Subject, VERSION},
};
use std::{collections::BTreeSet, path::Path, process::Command};

pub fn git(root: &Path, args: &[&str]) -> Result<Vec<u8>> {
    let output = crate::process::capture(
        Command::new("git")
            .arg("--no-optional-locks")
            .args([
                "-c",
                "core.fsmonitor=false",
                "-c",
                "core.hooksPath=/dev/null",
            ])
            .arg("-C")
            .arg(root)
            .args(args),
        std::time::Duration::from_secs(30),
    )?;
    if !output.status.success() {
        return Err(format!("git failed: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(output.stdout)
}
pub fn text(root: &Path, args: &[&str]) -> Result<String> {
    Ok(String::from_utf8(git(root, args)?)?
        .trim_end_matches('\n')
        .to_string())
}
fn names(bytes: &[u8]) -> Result<BTreeSet<String>> {
    bytes
        .split(|byte| *byte == 0)
        .filter(|part| !part.is_empty())
        .map(|part| {
            let name = String::from_utf8(part.to_vec())?;
            content::relative(&name)?;
            Ok(name)
        })
        .collect()
}
pub fn paths(root: &Path, base: &str, head: &str, worktree: bool) -> Result<Vec<String>> {
    let base = revision(root, base)?;
    let head = revision(root, head)?;
    let mut paths = names(&git(
        root,
        &[
            "diff",
            "--name-only",
            "--no-ext-diff",
            "--no-textconv",
            "--no-renames",
            "-z",
            &format!("{base}...{head}"),
            "--",
        ],
    )?)?;
    if worktree {
        paths.extend(names(&git(
            root,
            &[
                "diff",
                "--name-only",
                "--no-ext-diff",
                "--no-textconv",
                "--no-renames",
                "-z",
                &head,
                "--",
            ],
        )?)?);
        paths.extend(
            names(&git(
                root,
                &["ls-files", "--others", "--exclude-standard", "-z"],
            )?)?
            .into_iter()
            .filter(|p| !content::excluded(p)),
        );
    }
    Ok(paths.into_iter().collect())
}
fn revision(root: &Path, reference: &str) -> Result<String> {
    text(
        root,
        &[
            "rev-parse",
            "--verify",
            "--end-of-options",
            &format!("{reference}^{{commit}}"),
        ],
    )
}
pub fn committed(root: &Path, revision: &str, path: &str) -> Result<State> {
    content::relative(path)?;
    let entry = git(root, &["ls-tree", "-z", revision, "--", path])?;
    if entry.is_empty() {
        return Ok(State {
            path: path.into(),
            kind: "deleted".into(),
            digest: content::digest(&[]),
        });
    }
    let mode = entry
        .split(|byte| *byte == b' ')
        .next()
        .ok_or("missing tree mode")?;
    let kind = match mode {
        b"100644" | b"100755" => "file",
        b"120000" => "symlink",
        _ => return Err("unsupported Git tree entry (directory or submodule)".into()),
    };
    let bytes = git(root, &["cat-file", "blob", &format!("{revision}:{path}")])?;
    Ok(State {
        path: path.into(),
        kind: kind.into(),
        digest: content::digest(&bytes),
    })
}
fn inputs(root: &Path) -> Result<Vec<State>> {
    let flags = git(root, &["ls-files", "-v", "-z"])?;
    if flags
        .split(|byte| *byte == 0)
        .filter_map(|entry| entry.first())
        .any(|flag| flag.is_ascii_lowercase() || *flag == b'S')
    {
        return Err("hidden index flags forbid a complete epoch".into());
    }
    let mut paths = names(&git(root, &["ls-files", "--cached", "-z"])?)?;
    paths.extend(
        names(&git(
            root,
            &["ls-files", "--others", "--exclude-standard", "-z"],
        )?)?
        .into_iter()
        .filter(|p| !content::excluded(p)),
    );
    paths
        .into_iter()
        .map(|path| content::state(root, &path))
        .collect()
}
pub fn epoch(
    repository: &Path,
    base: &str,
    head: &str,
    worktree: bool,
    policy_root: &Path,
) -> Result<Epoch> {
    let root = Path::new(&text(repository, &["rev-parse", "--show-toplevel"])?).canonicalize()?;
    let repository = root.to_str().ok_or("non-UTF8 repository root")?.to_string();
    let base_sha = revision(&root, base)?;
    let head_sha = revision(&root, head)?;
    if head_sha != revision(&root, "HEAD")? {
        return Err("candidate head must be checked out".into());
    }
    let changed_paths = paths(&root, &base_sha, &head_sha, worktree)?;
    let path_states = changed_paths
        .iter()
        .map(|path| {
            if worktree {
                content::state(&root, path)
            } else {
                committed(&root, &head_sha, path)
            }
        })
        .collect::<Result<_>>()?;
    let subject = Subject {
        repository: repository.clone(),
        base_ref: base.into(),
        head_ref: head.into(),
        remote_url: text(
            &root,
            &["config", "--get-regexp", "^remote\\.origin\\.url$"],
        )
        .or_else(|error| {
            let remotes = text(&root, &["remote"])?;
            if remotes.lines().any(|remote| remote == "origin") {
                Err(error)
            } else {
                Ok(String::new())
            }
        })?,
        merge_base_sha: text(&root, &["merge-base", &base_sha, &head_sha])?,
        head_tree_sha: text(&root, &["rev-parse", &format!("{head_sha}^{{tree}}")])?,
        branch: text(&root, &["branch", "--show-current"])?,
        base_sha,
        head_sha,
        include_worktree: worktree,
        dirty_status_digest: dirty_digest(&root)?,
        submodule_status_digest: content::digest(&git(
            &root,
            &["submodule", "status", "--recursive"],
        )?),
        changed_paths,
        path_states,
        input_states: inputs(&root)?,
    };
    Ok(Epoch {
        schema_version: VERSION,
        repository,
        subject_digest: content::bound(&subject)?,
        policy_digest: content::bound(&content::policy(policy_root)?)?,
        classifier: classify(&subject.changed_paths)?,
        subject,
    })
}
pub fn current(record: &Epoch, policy_root: &Path) -> Result<()> {
    let expected = epoch(
        Path::new(&record.repository),
        &record.subject.base_ref,
        &record.subject.head_ref,
        record.subject.include_worktree,
        policy_root,
    )?;
    if *record != expected {
        return Err("epoch no longer matches current repository or policy".into());
    }
    Ok(())
}

fn dirty_digest(root: &Path) -> Result<String> {
    content::bound(&[
        git(
            root,
            &[
                "status",
                "--porcelain=v1",
                "-z",
                "--untracked-files=all",
                "--",
                ".",
                ":(exclude)**/.proof-integrity/**",
                ":(exclude).proof-integrity/**",
                ":(exclude)**/target/**",
            ],
        )?,
        git(root, &["ls-files", "--stage", "-z"])?,
        git(
            root,
            &[
                "diff",
                "--binary",
                "--full-index",
                "--no-ext-diff",
                "--no-textconv",
                "HEAD",
                "--",
            ],
        )?,
    ])
}
