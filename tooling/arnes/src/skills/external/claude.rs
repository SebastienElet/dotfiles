use super::manifest;
use super::model::{Exposure, Plugin, Topology, plugin_diagnostics};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::canonical_within;
use crate::manifest::{Agent, Manifest, Scope};
use std::collections::BTreeMap;
use std::fs;
use std::fs::DirEntry;
use std::path::Path;

mod registry;
mod settings;

pub(super) fn diagnose(roots: &Roots, policy: &Manifest, scope: Scope) -> Vec<Diagnostic> {
    let (settings, mut diagnostics) = settings::load(roots, scope);
    let mut plugins = skills_directory_plugins(roots, policy, scope, &settings.effective);
    let (installed, registry_diagnostics) = registry::diagnose(roots, scope, &settings.effective);
    plugins.extend(installed);
    diagnostics.extend(registry_diagnostics);
    for (id, enabled) in &settings.configured {
        if !plugins.iter().any(|plugin| &plugin.id == id) {
            plugins.push(Plugin {
                id: id.clone(),
                artifact: None,
                version: None,
                path: None,
                exposure: if *enabled {
                    Exposure::Enabled
                } else {
                    Exposure::Disabled
                },
                topology: Topology::Missing,
                detail: Some("plugin is configured without a registry installation".to_owned()),
                skills: Vec::new(),
            });
        }
    }
    if scope == Scope::User {
        let known = plugins
            .iter()
            .map(|plugin| plugin.id.as_str())
            .collect::<Vec<_>>();
        diagnostics.extend(super::cache_inventory::diagnose(
            roots,
            policy,
            Agent::Claude,
            &known,
        ));
    }
    plugins.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.version.cmp(&right.version))
    });
    diagnostics.extend(plugin_diagnostics(policy, Agent::Claude, scope, plugins));
    diagnostics
}

fn skills_directory_plugins(
    roots: &Roots,
    policy: &Manifest,
    scope: Scope,
    settings: &Result<BTreeMap<String, bool>, ()>,
) -> Vec<Plugin> {
    let mut plugins = Vec::new();
    for projection in policy
        .skill_projections()
        .filter(|projection| projection.agent == Agent::Claude && projection.scope == scope)
    {
        let base = match scope {
            Scope::User => roots.home(),
            Scope::Project => roots.repository(),
        };
        let directory = base.join(projection.destination);
        if canonical_within(&directory, base).is_none() {
            continue;
        }
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.filter_map(Result::ok) {
            if let Some(plugin) = inspect_skills_directory_entry(&directory, &entry, settings) {
                plugins.push(plugin);
            }
        }
    }
    plugins
}

fn inspect_skills_directory_entry(
    root: &Path,
    entry: &DirEntry,
    settings: &Result<BTreeMap<String, bool>, ()>,
) -> Option<Plugin> {
    let path = entry.path();
    canonical_within(&path, root)?;
    if !path.join(".claude-plugin/plugin.json").is_file() {
        return None;
    }
    let inspected = manifest::inspect(&path, &[".claude-plugin/plugin.json"]);
    let name = inspected
        .name
        .clone()
        .unwrap_or_else(|| entry.file_name().to_string_lossy().into_owned());
    let id = format!("{name}@skills-dir");
    Some(Plugin {
        exposure: settings::exposure(settings, &id),
        id,
        artifact: None,
        version: inspected.version,
        path: Some(path),
        topology: inspected.topology,
        detail: inspected.detail,
        skills: inspected.skills,
    })
}

fn installation_scope(scope: &str) -> Option<Scope> {
    match scope {
        "user" => Some(Scope::User),
        "project" | "local" => Some(Scope::Project),
        _ => None,
    }
}

fn registry_diagnostic(
    scope: Scope,
    subject: &str,
    path: &Path,
    state: State,
    topology: &str,
    detail: &str,
) -> Diagnostic {
    Diagnostic::new(
        "skills",
        state,
        format!(
            "external claude {scope} {subject} origin=plugin ownership=external exposure=unknown topology={topology} policy=unknown activation=unknown path={} detail={detail}",
            path.display(),
        ),
    )
}
