use super::{Plugin, Skill, SystemSkill, Topology};
use crate::manifest::{Agent, Scope};
use std::path::Path;

pub(super) fn plugin_group(agent: Agent, scope: Scope, plugin: &Plugin) -> String {
    format!(
        "{agent} {scope} plugin {}@{}",
        plugin.id,
        plugin
            .version
            .as_deref()
            .or(plugin.artifact.as_deref())
            .unwrap_or("?")
    )
}

pub(super) fn plugin_summary(plugin: &Plugin, policy: &str) -> String {
    format!(
        "plugin · {} · {} · {policy}{}{}{}",
        plugin.exposure,
        plugin.topology,
        artifact(plugin.artifact.as_deref()),
        path(plugin.topology, plugin.path.as_deref()),
        detail(plugin.detail.as_deref()),
    )
}

pub(super) fn plugin_skill_summary(plugin: &Plugin, skill: &Skill, policy: &str) -> String {
    format!(
        "skill {} · {} · {} · {policy}{}{}",
        skill.slug,
        plugin.exposure,
        skill.topology,
        path(skill.topology, Some(&skill.path)),
        detail(skill.detail.as_deref()),
    )
}

pub(super) fn system_skill_summary(skill: &SystemSkill, policy: &str) -> String {
    format!(
        "{} · {} · {} · {policy}{}{}",
        skill.slug,
        skill.exposure,
        skill.topology,
        path(skill.topology, Some(&skill.path)),
        detail(skill.detail.as_deref()),
    )
}

fn detail(value: Option<&str>) -> String {
    value.map_or_else(String::new, |value| format!(" — {value}"))
}

fn artifact(value: Option<&str>) -> String {
    value.map_or_else(String::new, |value| format!(" · artifact={value}"))
}

fn path(topology: Topology, path: Option<&Path>) -> String {
    if topology == Topology::Healthy {
        String::new()
    } else {
        path.map_or_else(String::new, |path| format!(" · {}", path.display()))
    }
}
