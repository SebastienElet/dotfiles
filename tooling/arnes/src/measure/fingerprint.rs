mod plugins;
use super::MeasureError;
use super::model::HookAgent;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

const MAX_FILES: usize = 512;
const MAX_FILE_BYTES: u64 = 1_048_576;

pub struct Fingerprint {
    pub digest: String,
    pub limitations: Vec<String>,
}

pub fn deployed(
    agent: HookAgent,
    home: &Path,
    repository: &Path,
) -> Result<Fingerprint, MeasureError> {
    let mut hasher = Sha256::new();
    let mut seen = HashSet::new();
    let mut remaining = MAX_FILES;
    let mut selected = selected_roots(agent, home, repository);
    let plugins = plugins::selected(agent, home, repository)?;
    hash_markers(&plugins.markers, &mut hasher, &mut remaining)?;
    selected.extend(plugins.roots);
    for root in selected {
        let result = hash_selected(&root, &mut hasher, &mut seen, &mut remaining);
        if let Err(error) = result {
            if error.kind() == std::io::ErrorKind::InvalidData {
                return Err(MeasureError::new(error.to_string()));
            }
            write!(hasher, "unreadable\0{:?}\0", error.kind()).map_err(MeasureError::from)?;
        }
    }
    Ok(Fingerprint {
        digest: format!("{:x}", hasher.finalize()),
        limitations: plugins.limitations,
    })
}

struct SelectedRoot {
    label: PathBuf,
    path: PathBuf,
    bounded: bool,
}

fn selected_roots(agent: HookAgent, home: &Path, repository: &Path) -> Vec<SelectedRoot> {
    let mut selected = Vec::new();
    selected.extend(home_roots(agent).iter().map(|path| SelectedRoot {
        label: Path::new("home").join(path),
        path: home.join(path),
        bounded: false,
    }));
    selected.extend(repository_roots(agent).iter().map(|path| SelectedRoot {
        label: Path::new("repository").join(path),
        path: repository.join(path),
        bounded: false,
    }));
    selected
}

fn home_roots(agent: HookAgent) -> &'static [&'static str] {
    match agent {
        HookAgent::Codex => &[
            ".codex/config.toml",
            ".codex/AGENTS.md",
            ".codex/hooks.json",
            ".agents/skills",
            ".codex/skills/.system",
        ],
        HookAgent::ClaudeCode => &[
            ".claude/settings.json",
            ".claude/CLAUDE.md",
            ".claude/SOUL.md",
            ".claude/USER.md",
            ".claude/hooks",
            ".claude/rules",
            ".claude/skills",
        ],
        HookAgent::Cursor => &[
            ".cursor/cli-config.json",
            ".arnes.yaml",
            ".cursor/hooks.json",
            ".cursor/rules",
            ".cursor/skills",
        ],
    }
}

fn repository_roots(agent: HookAgent) -> &'static [&'static str] {
    match agent {
        HookAgent::Codex => &[
            "AGENTS.md",
            ".codex/config.toml",
            ".codex/hooks.json",
            ".codex/skills",
        ],
        HookAgent::ClaudeCode => &[
            "AGENTS.md",
            "CLAUDE.md",
            ".claude/settings.json",
            ".claude/settings.local.json",
            ".claude/hooks",
            ".claude/skills",
        ],
        HookAgent::Cursor => &[
            "AGENTS.md",
            ".cursor/cli.json",
            ".cursor/hooks.json",
            ".cursor/rules",
            ".cursor/skills",
        ],
    }
}

fn hash_path(
    path: &Path,
    label: &Path,
    hasher: &mut Sha256,
    seen: &mut HashSet<PathBuf>,
    remaining: &mut usize,
    boundary: Option<&Path>,
) -> std::io::Result<()> {
    write!(hasher, "{}\0", label.display())?;
    if !path.exists() {
        hasher.update(b"missing\0");
        return Ok(());
    }
    consume_entry(remaining)?;
    let canonical = fs::canonicalize(path)?;
    if boundary.is_some_and(|boundary| !canonical.starts_with(boundary)) {
        hasher.update(b"escape\0");
        return Ok(());
    }
    if !seen.insert(canonical) {
        hasher.update(b"cycle\0");
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        hash_directory(path, label, hasher, seen, remaining, boundary)
    } else if metadata.is_file() {
        hash_file(path, metadata.len(), hasher)
    } else {
        hasher.update(b"unsupported\0");
        Ok(())
    }
}

fn hash_directory(
    path: &Path,
    label: &Path,
    hasher: &mut Sha256,
    seen: &mut HashSet<PathBuf>,
    remaining: &mut usize,
    boundary: Option<&Path>,
) -> std::io::Result<()> {
    hasher.update(b"directory\0");
    let limit = *remaining;
    let mut entries = fs::read_dir(path)?
        .take(limit.saturating_add(1))
        .collect::<Result<Vec<_>, _>>()?;
    if entries.len() > limit {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "fingerprint inventory exceeds 512 entries",
        ));
    }
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        hash_path(
            &entry.path(),
            &label.join(entry.file_name()),
            hasher,
            seen,
            remaining,
            boundary,
        )?;
    }
    Ok(())
}

fn hash_file(path: &Path, size: u64, hasher: &mut Sha256) -> std::io::Result<()> {
    if size > MAX_FILE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "fingerprint file exceeds 1048576 bytes",
        ));
    }
    write!(hasher, "file\0{size}\0")?;
    let mut file = File::open(path)?.take(MAX_FILE_BYTES + 1);
    if std::io::copy(&mut file, hasher)? > MAX_FILE_BYTES {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "fingerprint file exceeds 1048576 bytes",
        ));
    }
    Ok(())
}

fn hash_selected(
    root: &SelectedRoot,
    hasher: &mut Sha256,
    seen: &mut HashSet<PathBuf>,
    remaining: &mut usize,
) -> std::io::Result<()> {
    let boundary = if root.bounded && root.path.exists() {
        Some(fs::canonicalize(&root.path)?)
    } else {
        None
    };
    hash_path(
        &root.path,
        &root.label,
        hasher,
        seen,
        remaining,
        boundary.as_deref(),
    )
}

fn consume_entry(remaining: &mut usize) -> std::io::Result<()> {
    if *remaining == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "fingerprint inventory exceeds 512 entries",
        ));
    }
    *remaining -= 1;
    Ok(())
}

fn hash_markers(
    markers: &[String],
    hasher: &mut Sha256,
    remaining: &mut usize,
) -> Result<(), MeasureError> {
    for marker in markers {
        consume_entry(remaining).map_err(MeasureError::from)?;
        write!(hasher, "plugin-state\0{marker}\0").map_err(MeasureError::from)?;
    }
    Ok(())
}
