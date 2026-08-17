use super::codex;
use super::model::{Exposure, SystemSkill, Topology, external_skill_diagnostic};
use super::unsupported;
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within, label};
use crate::manifest::{Agent, ExternalOrigin, Manifest, Scope};
use std::collections::BTreeMap;
use std::fs::{self, DirEntry};
use std::io::ErrorKind;
use std::path::Path;

pub(super) fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Vec<Diagnostic> {
    let declared = manifest
        .external_roots()
        .filter(|root| root.agent == agent && root.scope == scope)
        .collect::<Vec<_>>();
    if declared.is_empty() {
        return vec![unsupported(
            agent,
            scope,
            "system skills inventory is unsupported",
        )];
    }
    let base = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let mut diagnostics = Vec::new();
    let mut skills = Vec::new();
    for root in declared {
        let (root_diagnostics, root_skills) = scan_root(roots, agent, scope, root.path, base);
        diagnostics.extend(root_diagnostics);
        skills.extend(root_skills);
    }
    let mut counts = BTreeMap::new();
    for skill in &skills {
        *counts.entry(skill.slug.clone()).or_insert(0) += 1;
    }
    for mut skill in skills {
        if counts[&skill.slug] > 1 {
            skill.topology = Topology::Broken;
            skill.detail = Some("duplicate system skill slug across roots".to_owned());
        }
        let allowed = manifest.external_skills(agent, scope).any(|allowed| {
            allowed.origin == ExternalOrigin::System
                && allowed.plugin.is_none()
                && allowed.slug == skill.slug
        });
        diagnostics.push(external_skill_diagnostic(agent, scope, skill, allowed));
    }
    diagnostics
}

fn scan_root(
    roots: &Roots,
    agent: Agent,
    scope: Scope,
    relative: &Path,
    base: &Path,
) -> (Vec<Diagnostic>, Vec<SystemSkill>) {
    let directory = base.join(relative);
    let entries = match root_entries(&directory, base) {
        Ok(Some(entries)) => entries,
        Ok(None) => {
            return (
                vec![root_diagnostic(
                    agent,
                    scope,
                    relative,
                    State::Healthy,
                    "absent",
                    "healthy",
                    "root is absent",
                )],
                Vec::new(),
            );
        }
        Err((topology, detail)) => {
            return (
                vec![root_diagnostic(
                    agent,
                    scope,
                    relative,
                    State::Error,
                    "unknown",
                    topology,
                    detail,
                )],
                Vec::new(),
            );
        }
    };
    let skills = entries
        .into_iter()
        .filter_map(|entry| entry_skill(roots, agent, &directory, entry))
        .collect();
    (Vec::new(), skills)
}

fn root_entries(
    directory: &Path,
    base: &Path,
) -> Result<Option<Vec<DirEntry>>, (&'static str, &'static str)> {
    let metadata = match fs::symlink_metadata(directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return if ancestor_within(directory, base) {
                Ok(None)
            } else {
                Err(("broken", "root has an ancestor outside its scope"))
            };
        }
        Err(_) => return Err(("unreadable", "root metadata could not be read")),
    };
    if canonical_within(directory, base).is_none() {
        let detail = if metadata.file_type().is_symlink() && fs::metadata(directory).is_err() {
            "root symlink is dangling"
        } else {
            "root resolves outside its scope"
        };
        return Err(("broken", detail));
    }
    if !fs::metadata(directory).is_ok_and(|metadata| metadata.is_dir()) {
        return Err(("broken", "root is not a directory"));
    }
    let mut entries = fs::read_dir(directory)
        .map_err(|_| ("unreadable", "root directory could not be read"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ("unreadable", "root directory entry could not be read"))?;
    entries.sort_by_key(|entry| entry.file_name());
    Ok(Some(entries))
}

fn entry_skill(roots: &Roots, agent: Agent, root: &Path, entry: DirEntry) -> Option<SystemSkill> {
    let slug = entry.file_name().to_string_lossy().into_owned();
    let path = entry.path();
    let (topology, detail, exposure) = match entry.file_type() {
        Ok(kind) if kind.is_dir() || kind.is_symlink() => {
            let (topology, detail) = classify_skill(&path, root);
            let exposure = match agent {
                Agent::Codex => codex::skill_exposure(roots, &path.join("SKILL.md")),
                Agent::Claude | Agent::Cursor => Exposure::Enabled,
            };
            (topology, detail, exposure)
        }
        Ok(_) => return None,
        Err(_) => (
            Topology::Unreadable,
            Some("skill metadata could not be read"),
            Exposure::Unknown,
        ),
    };
    Some(SystemSkill {
        slug,
        path,
        exposure,
        topology,
        detail: detail.map(str::to_owned),
    })
}

fn classify_skill(path: &Path, root: &Path) -> (Topology, Option<&'static str>) {
    match fs::metadata(path) {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return (Topology::Broken, Some("skill link is dangling"));
        }
        Err(_) => {
            return (
                Topology::Unreadable,
                Some("skill metadata could not be read"),
            );
        }
        Ok(metadata) if !metadata.is_dir() => {
            return (Topology::Broken, Some("skill is not a directory"));
        }
        Ok(_) => {}
    }
    if canonical_within(path, root).is_none() {
        return (Topology::Broken, Some("skill resolves outside its root"));
    }
    let skill_file = path.join("SKILL.md");
    match fs::metadata(&skill_file) {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return (Topology::Broken, Some("SKILL.md is missing"));
        }
        Err(_) => {
            return (
                Topology::Unreadable,
                Some("SKILL.md metadata could not be read"),
            );
        }
        Ok(metadata) if !metadata.is_file() => {
            return (Topology::Broken, Some("SKILL.md is not a file"));
        }
        Ok(_) => {}
    }
    if canonical_within(&skill_file, root).is_none() {
        return (Topology::Broken, Some("SKILL.md resolves outside its root"));
    }
    (Topology::Healthy, None)
}

fn root_diagnostic(
    agent: Agent,
    scope: Scope,
    root: &Path,
    state: State,
    exposure: &str,
    topology: &str,
    detail: &str,
) -> Diagnostic {
    Diagnostic::new(
        "skills",
        state,
        format!(
            "external {agent} {scope} system skills root {} origin=system ownership=external exposure={exposure} topology={topology} policy=not-applicable activation=not-runtime-observed detail={detail}",
            label(scope, root),
        ),
    )
}
