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
    let plugins = plugins::selected(agent, home)?;
    selected.extend(plugins.roots);
    for root in selected {
        let result = hash_path(
            &root.path,
            &root.label,
            &mut hasher,
            &mut seen,
            &mut remaining,
        );
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
}

fn selected_roots(agent: HookAgent, home: &Path, repository: &Path) -> Vec<SelectedRoot> {
    let mut selected = Vec::new();
    selected.extend(home_roots(agent).iter().map(|path| SelectedRoot {
        label: Path::new("home").join(path),
        path: home.join(path),
    }));
    selected.extend(repository_roots(agent).iter().map(|path| SelectedRoot {
        label: Path::new("repository").join(path),
        path: repository.join(path),
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
            ".claude/plugins/installed_plugins.json",
        ],
        HookAgent::Cursor => &[
            ".cursor/cli-config.json",
            ".cursor/hooks.json",
            ".cursor/rules",
            ".cursor/skills",
            ".cursor/plugins/local",
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
) -> std::io::Result<()> {
    write!(hasher, "{}\0", label.display())?;
    if !path.exists() {
        hasher.update(b"missing\0");
        return Ok(());
    }
    consume_entry(remaining)?;
    let canonical = fs::canonicalize(path)?;
    if !seen.insert(canonical) {
        hasher.update(b"cycle\0");
        return Ok(());
    }
    let metadata = fs::metadata(path)?;
    if metadata.is_dir() {
        hash_directory(path, label, hasher, seen, remaining)
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
        )?;
    }
    Ok(())
}

fn hash_file(path: &Path, size: u64, hasher: &mut Sha256) -> std::io::Result<()> {
    write!(hasher, "file\0{size}\0")?;
    let mut file = File::open(path)?.take(MAX_FILE_BYTES);
    std::io::copy(&mut file, hasher)?;
    Ok(())
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
