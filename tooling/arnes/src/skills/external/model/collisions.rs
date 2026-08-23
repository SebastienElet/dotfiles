use super::{Plugin, Topology};
use std::collections::BTreeMap;

pub(super) fn mark(plugins: &mut [Plugin]) {
    let mut plugin_counts = BTreeMap::new();
    let mut skill_counts = BTreeMap::new();
    for plugin in plugins.iter() {
        *plugin_counts.entry(plugin.id.clone()).or_insert(0) += 1;
        for skill in &plugin.skills {
            *skill_counts
                .entry((plugin.id.clone(), skill.slug.clone()))
                .or_insert(0) += 1;
        }
    }
    for plugin in plugins {
        if plugin_counts.get(&plugin.id).copied().unwrap_or(0) > 1 {
            mark_one(
                &mut plugin.topology,
                &mut plugin.detail,
                "duplicate plugin identifier",
            );
        }
        for skill in &mut plugin.skills {
            if skill_counts
                .get(&(plugin.id.clone(), skill.slug.clone()))
                .copied()
                .unwrap_or(0)
                > 1
            {
                mark_one(
                    &mut skill.topology,
                    &mut skill.detail,
                    "duplicate plugin skill slug",
                );
            }
        }
    }
}

fn mark_one(topology: &mut Topology, detail: &mut Option<String>, reason: &str) {
    if *topology != Topology::Unreadable {
        *topology = Topology::Broken;
    }
    *detail = Some(match detail.take() {
        Some(detail) => format!("{detail}; {reason}"),
        None => reason.to_owned(),
    });
}
