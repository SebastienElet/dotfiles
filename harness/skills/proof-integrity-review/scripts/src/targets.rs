use crate::repository;
use crate::{
    Result, content,
    model::{Epoch, Target},
};
#[cfg(unix)]
use std::os::unix::fs::MetadataExt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub fn targets(
    epoch: &Epoch,
    policy_root: &Path,
    policy: &BTreeMap<String, String>,
    targets: &[Target],
) -> Result<BTreeSet<String>> {
    if targets.is_empty() || targets.len() > 64 {
        return Err("mutation requires 1..64 targets".into());
    }
    let mut identities = BTreeSet::new();
    let mut sources = BTreeSet::new();
    let mut physical = BTreeSet::new();
    let mut scopes = BTreeSet::new();
    let mut bytes = 0_usize;
    for target in targets {
        content::target_path(&target.path)?;
        if !identities.insert((&target.scope, &target.path)) {
            return Err("duplicate mutation target".into());
        }
        scopes.insert(target.scope.clone());
        bytes = bytes
            .checked_add(target.before_content.len())
            .and_then(|n| n.checked_add(target.after_content.len()))
            .ok_or("mutation size overflow")?;
        if bytes > 8 * 1024 * 1024 {
            return Err("mutation content exceeds 8MiB".into());
        }
        if content::digest(target.before_content.as_bytes()) != target.before_digest
            || content::digest(target.after_content.as_bytes()) != target.after_digest
            || target.before_digest == target.after_digest
        {
            return Err("mutation content digest mismatch or unchanged target".into());
        }
        if let Some(source) = baseline(epoch, policy_root, policy, target)? {
            let canonical = source.canonicalize()?;
            if !sources.insert(canonical) {
                return Err("mutation target file alias".into());
            }
            let metadata = fs::metadata(source)?;
            #[cfg(unix)]
            if !physical.insert((metadata.dev(), metadata.ino())) {
                return Err("mutation target physical alias".into());
            }
        }
    }
    Ok(scopes)
}
fn baseline(
    epoch: &Epoch,
    policy_root: &Path,
    policy: &BTreeMap<String, String>,
    target: &Target,
) -> Result<Option<std::path::PathBuf>> {
    let source = match target.scope.as_str() {
        "fixture" => return Ok(None),
        "repository" => {
            let root = Path::new(&epoch.repository);
            let baseline = if epoch.subject.include_worktree {
                content::state(root, &target.path)?
            } else {
                repository::committed(root, &epoch.subject.head_sha, &target.path)?
            };
            if !epoch
                .subject
                .input_states
                .iter()
                .any(|state| state.path == target.path)
                || baseline.kind != "file"
                || baseline.digest != target.before_digest
            {
                return Err("repository target baseline differs from frozen candidate".into());
            }
            content::safe_source(Path::new(&epoch.repository), &target.path)?
        }
        "policy" => {
            if policy.get(&target.path) != Some(&target.before_digest) {
                return Err("policy target absent or mismatched".into());
            }
            content::safe_source(policy_root, &target.path)?
        }
        _ => return Err("unsupported mutation scope".into()),
    };
    if content::digest(&fs::read(&source)?) != target.before_digest {
        return Err("mutation target changed since epoch".into());
    }
    Ok(Some(source))
}
