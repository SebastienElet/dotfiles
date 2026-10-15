use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within};
use crate::inspection::quote;
use crate::manifest::{Agent, Manifest, Scope};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn removal(agent: Agent, scope: Scope, id: &str) -> String {
    let id = quote(id);
    match agent {
        Agent::Claude => format!("claude plugin uninstall {id} --scope {scope}"),
        Agent::Codex => format!("codex plugin remove {id}"),
        Agent::Cursor => {
            format!("Cursor Settings > Plugins: uninstall {id} (no native CLI removal)")
        }
    }
}

pub(super) fn diagnose(
    roots: &Roots,
    policy: &Manifest,
    agent: Agent,
    known: &[&str],
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let allowed = policy
        .external_plugins(agent, Scope::User)
        .chain(policy.external_plugins(agent, Scope::Project))
        .collect::<Vec<_>>();
    let cache = roots.home().join(format!(".{agent}/plugins/cache"));
    for marketplace in directories(&cache, roots.home(), &mut diagnostics) {
        let name = marketplace
            .file_name()
            .unwrap_or_default()
            .to_string_lossy();
        let declared = allowed.iter().any(|id| {
            id.rsplit_once('@')
                .is_some_and(|(_, market)| market == name)
        });
        if !declared {
            diagnostics.push(Diagnostic::new("skills", State::Drift, format!("{agent} user marketplace {name}: undeclared cache {}; proposed removal: {agent} plugin marketplace remove {} (orphan cache may require manual removal of this exact directory)", marketplace.display(), quote(&name))));
        }
        for plugin in directories(&marketplace, roots.home(), &mut diagnostics) {
            let id = format!(
                "{}@{name}",
                plugin.file_name().unwrap_or_default().to_string_lossy()
            );
            if !allowed.contains(&id.as_str()) && !known.contains(&id.as_str()) {
                diagnostics.push(Diagnostic::new("skills", State::Drift, format!("{agent} user plugin {id}: undeclared cache {}; proposed removal: {} (orphan cache may require manual removal of this exact directory)", plugin.display(), removal(agent, Scope::User, &id))));
            }
        }
    }
    diagnostics
}

fn directories(path: &Path, home: &Path, diagnostics: &mut Vec<Diagnostic>) -> Vec<PathBuf> {
    let result = (|| {
        match fs::symlink_metadata(path) {
            Err(error)
                if error.kind() == std::io::ErrorKind::NotFound && ancestor_within(path, home) =>
            {
                return Ok(Vec::new());
            }
            Err(_) => return Err("cache metadata could not be read"),
            Ok(_) if canonical_within(path, home).is_none() => {
                return Err("cache resolves outside HOME");
            }
            Ok(_) => {}
        }
        let mut directories = Vec::new();
        for entry in fs::read_dir(path).map_err(|_| "cache directory could not be read")? {
            let entry = entry.map_err(|_| "cache entry could not be read")?;
            if canonical_within(&entry.path(), home).is_none() {
                return Err("cache entry resolves outside HOME");
            }
            if entry.path().is_dir() {
                directories.push(entry.path());
            }
        }
        directories.sort();
        Ok(directories)
    })();
    result.unwrap_or_else(|error| {
        diagnostics.push(Diagnostic::new(
            "skills",
            State::Error,
            format!("{}: {error}", path.display()),
        ));
        Vec::new()
    })
}
