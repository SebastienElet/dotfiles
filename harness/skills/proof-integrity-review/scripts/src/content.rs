use crate::{Result, model::State};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs,
    path::{Component, Path},
};

pub fn digest(bytes: &[u8]) -> String {
    format!("sha256:{:x}", Sha256::digest(bytes))
}
pub fn bound(value: &impl Serialize) -> Result<String> {
    Ok(digest(&serde_json::to_vec(&serde_json::to_value(value)?)?))
}
pub fn relative(path: &str) -> Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.contains('\\')
        || path.contains(':')
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return Err("path must be canonical and relative".into());
    }
    Ok(())
}
pub fn target_path(path: &str) -> Result<()> {
    relative(path)?;
    if path.trim() != path || path.chars().any(char::is_control) {
        return Err("mutation path contains whitespace or control characters".into());
    }
    Ok(())
}
pub fn safe_source(root: &Path, path: &str) -> Result<std::path::PathBuf> {
    relative(path)?;
    let mut current = root.to_path_buf();
    for part in Path::new(path).components() {
        if let Component::Normal(name) = part {
            current.push(name);
        } else {
            return Err("non-canonical source path".into());
        }
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_symlink() => return Err("symlink source refused".into()),
            Ok(_) => (),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(current)
}
pub fn state(root: &Path, path: &str) -> Result<State> {
    relative(path)?;
    let source = root.join(path);
    if let Some(parent) = Path::new(path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        safe_source(root, parent.to_str().ok_or("non-UTF8 parent")?)?;
    }
    let (kind, bytes) = match fs::symlink_metadata(&source) {
        Ok(metadata) if metadata.is_symlink() => (
            "symlink",
            fs::read_link(source)?
                .as_os_str()
                .as_encoded_bytes()
                .to_vec(),
        ),
        Ok(metadata) if metadata.is_file() => ("file", fs::read(source)?),
        Ok(_) => return Err(format!("unsupported repository input: {path}").into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => ("deleted", Vec::new()),
        Err(error) => return Err(error.into()),
    };
    Ok(State {
        path: path.into(),
        kind: kind.into(),
        digest: digest(&bytes),
    })
}
pub fn excluded(path: &str) -> bool {
    path.split('/')
        .any(|part| matches!(part, ".git" | "target" | ".proof-integrity"))
}
pub fn policy(root: &Path) -> Result<BTreeMap<String, String>> {
    if fs::symlink_metadata(root)?.is_symlink() {
        return Err("policy root is a symlink".into());
    }
    let mut files = BTreeMap::new();
    collect_policy(root, root, &mut files)?;
    let required_sources: Vec<String> =
        serde_json::from_str(include_str!("../policy-sources.json"))?;
    for required in required_sources {
        if !files.contains_key(&required) {
            return Err(format!("missing policy source: {required}").into());
        }
    }
    Ok(files)
}
fn collect_policy(
    root: &Path,
    directory: &Path,
    files: &mut BTreeMap<String, String>,
) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let full = entry.path();
        let path = full
            .strip_prefix(root)?
            .to_str()
            .ok_or("non-UTF8 policy path")?
            .to_string();
        if excluded(&path) {
            continue;
        }
        let metadata = fs::symlink_metadata(&full)?;
        if metadata.is_symlink() {
            return Err(format!("symlink policy source: {path}").into());
        }
        if metadata.is_dir() {
            collect_policy(root, &full, files)?;
        } else if metadata.is_file() {
            files.insert(path, digest(&fs::read(full)?));
        } else {
            return Err("non-regular policy source".into());
        }
    }
    Ok(())
}
