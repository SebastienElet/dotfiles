use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, ExternalOrigin, Manifest, Scope};
use std::fmt::{self, Display};
use std::path::PathBuf;

mod collisions;
mod human;

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Exposure {
    Enabled,
    Disabled,
    Unknown,
}

impl Display for Exposure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
            Self::Unknown => "unknown",
        })
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Topology {
    Healthy,
    Broken,
    Unreadable,
    Unknown,
}

impl Display for Topology {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Healthy => "healthy",
            Self::Broken => "broken",
            Self::Unreadable => "unreadable",
            Self::Unknown => "unknown",
        })
    }
}

pub(super) struct Plugin {
    pub id: String,
    pub version: Option<String>,
    pub path: Option<PathBuf>,
    pub exposure: Exposure,
    pub topology: Topology,
    pub detail: Option<String>,
    pub skills: Vec<Skill>,
}

pub(super) struct Skill {
    pub slug: String,
    pub path: PathBuf,
    pub topology: Topology,
    pub detail: Option<String>,
}

pub(super) struct SystemSkill {
    pub slug: String,
    pub path: PathBuf,
    pub exposure: Exposure,
    pub topology: Topology,
    pub detail: Option<String>,
}

pub(super) fn plugin_diagnostics(
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
    mut plugins: Vec<Plugin>,
) -> Vec<Diagnostic> {
    collisions::mark(&mut plugins);
    plugins.sort_by(|left, right| {
        left.id
            .cmp(&right.id)
            .then_with(|| left.version.cmp(&right.version))
            .then_with(|| left.path.cmp(&right.path))
    });
    let allowed = manifest.external_plugins(agent, scope).collect::<Vec<_>>();
    let allowed_skills = manifest
        .external_skills(agent, scope)
        .filter(|skill| skill.origin == ExternalOrigin::Plugin)
        .collect::<Vec<_>>();
    plugins
        .into_iter()
        .flat_map(|plugin| {
            let plugin_allowed = allowed.contains(&plugin.id.as_str());
            let mut diagnostics = vec![plugin_diagnostic(agent, scope, &plugin, plugin_allowed)];
            diagnostics.extend(plugin.skills.iter().map(|skill| {
                let skill_allowed = plugin_allowed
                    && allowed_skills.iter().any(|allowed| {
                        allowed.plugin == Some(plugin.id.as_str()) && allowed.slug == skill.slug
                    });
                skill_diagnostic(agent, scope, &plugin, skill, skill_allowed)
            }));
            diagnostics
        })
        .collect()
}

fn plugin_diagnostic(agent: Agent, scope: Scope, plugin: &Plugin, allowed: bool) -> Diagnostic {
    Diagnostic::new(
        "skills",
        state(plugin.topology, plugin.exposure, allowed),
        format!(
            "external {agent} {scope} plugin {} origin=plugin ownership=external version={} exposure={} topology={} policy={} activation={} path={}{}",
            plugin.id,
            plugin.version.as_deref().unwrap_or("unknown"),
            plugin.exposure,
            plugin.topology,
            policy(allowed),
            activation(plugin.exposure),
            plugin
                .path
                .as_deref()
                .map_or_else(|| "unknown".to_owned(), |path| path.display().to_string()),
            detail(plugin.detail.as_deref()),
        ),
    )
    .with_human(
        human::plugin_group(agent, scope, plugin),
        human::plugin_summary(plugin, policy(allowed)),
    )
}

fn skill_diagnostic(
    agent: Agent,
    scope: Scope,
    plugin: &Plugin,
    skill: &Skill,
    allowed: bool,
) -> Diagnostic {
    Diagnostic::new(
        "skills",
        state(skill.topology, plugin.exposure, allowed),
        format!(
            "external {agent} {scope} skill {} origin=plugin ownership=external container={} version={} exposure={} topology={} policy={} activation={} path={}{}",
            skill.slug,
            plugin.id,
            plugin.version.as_deref().unwrap_or("unknown"),
            plugin.exposure,
            skill.topology,
            policy(allowed),
            activation(plugin.exposure),
            skill.path.display(),
            detail(skill.detail.as_deref()),
        ),
    )
    .with_human(
        human::plugin_group(agent, scope, plugin),
        human::plugin_skill_summary(plugin, skill, policy(allowed)),
    )
}

pub(super) fn external_skill_diagnostic(
    agent: Agent,
    scope: Scope,
    skill: SystemSkill,
    allowed: bool,
) -> Diagnostic {
    Diagnostic::new(
        "skills",
        state(skill.topology, skill.exposure, allowed),
        format!(
            "external {agent} {scope} skill {} origin=system ownership=external container=none version=unknown exposure={} topology={} policy={} activation={} path={}{}",
            skill.slug,
            skill.exposure,
            skill.topology,
            policy(allowed),
            activation(skill.exposure),
            skill.path.display(),
            detail(skill.detail.as_deref()),
        ),
    )
    .with_human(
        format!("{agent} {scope} system skills"),
        human::system_skill_summary(&skill, policy(allowed)),
    )
}

fn state(topology: Topology, exposure: Exposure, allowed: bool) -> State {
    match (topology, exposure, allowed) {
        (Topology::Broken | Topology::Unreadable, _, _) => State::Error,
        (_, Exposure::Enabled, false) => State::Drift,
        (Topology::Unknown, _, _) | (_, Exposure::Unknown, _) => State::Unsupported,
        _ => State::Healthy,
    }
}

fn policy(allowed: bool) -> &'static str {
    if allowed { "allowed" } else { "unexpected" }
}

fn activation(exposure: Exposure) -> &'static str {
    match exposure {
        Exposure::Enabled => "available-not-runtime-observed",
        Exposure::Disabled => "disabled",
        Exposure::Unknown => "unknown",
    }
}

fn detail(value: Option<&str>) -> String {
    value.map_or_else(String::new, |value| format!(" detail={value}"))
}
