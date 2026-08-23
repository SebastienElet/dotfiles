mod plugins;
mod traversal;
use super::MeasureError;
use super::model::HookAgent;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use traversal::Traversal;

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
    let mut selected = selected_roots(agent, home, repository);
    let plugins = plugins::selected(agent, home, repository)?;
    let mut limitations = plugins.limitations;
    let mut traversal = Traversal::new(&mut hasher, &mut limitations);
    traversal.hash_markers(&plugins.markers)?;
    selected.extend(plugins.roots);
    for root in selected {
        traversal.hash_selected(&root)?;
    }
    Ok(Fingerprint {
        digest: format!("{:x}", hasher.finalize()),
        limitations,
    })
}

pub(super) struct SelectedRoot {
    pub(super) label: PathBuf,
    pub(super) path: PathBuf,
    pub(super) bounded: bool,
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
