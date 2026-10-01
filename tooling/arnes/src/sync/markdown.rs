use super::{SyncEntry, SyncResource, SyncState};
use crate::Roots;
use crate::diagnostic::State;
use crate::manifest::{Agent, Manifest, Scope};
use crate::prompts::ProjectionTracker;

pub(super) mod owned;
mod publication;
mod selection;

pub(super) fn synchronize(
    roots: &Roots,
    manifest: &Manifest,
    resource: SyncResource,
    agent: Agent,
    scope: Scope,
) -> Vec<SyncEntry> {
    if !manifest.combinations().any(|pair| pair == (agent, scope))
        || !supported(resource, agent, scope)
    {
        return vec![SyncEntry::new(
            "selection",
            SyncState::Unsupported,
            "Markdown agent or scope is unsupported",
        )];
    }
    let selected = if resource == SyncResource::Prompts {
        selection::prompts(manifest, agent, scope)
    } else {
        match selection::commands(manifest, agent, scope) {
            Ok(selected) => selected,
            Err(message) => return vec![SyncEntry::new("selection", SyncState::Refused, message)],
        }
    };
    if selected.is_empty() {
        return vec![SyncEntry::new(
            "selection",
            SyncState::Empty,
            "no managed Markdown projections selected",
        )];
    }
    let scopes = ProjectionTracker::relevant_scopes(roots, &[scope]);
    let mut topology = ProjectionTracker::new_for_scopes(roots, manifest, &scopes);
    for prompt in manifest.prompts() {
        for projection in prompt
            .projections()
            .filter(|projection| scopes.contains(&projection.scope))
        {
            if !selected.iter().any(|candidate| {
                candidate.prompt.id() == prompt.id()
                    && candidate.projection.agent == projection.agent
                    && candidate.projection.scope == projection.scope
            }) {
                topology.seed_projection_destination(roots, prompt, projection);
            }
        }
    }
    let mut prepared = Vec::new();
    let mut refused = Vec::new();
    for candidate in selected {
        let result = topology
            .validate(roots, candidate.prompt, candidate.projection)
            .map_err(|_| "Markdown source or destination collides with another resource")
            .and_then(|()| selection::contents(roots, manifest, &candidate));
        match result {
            Ok(contents) => match publication::prepare(roots, manifest, &candidate, contents) {
                Ok(projection) => prepared.push((candidate, projection)),
                Err(entry) => refused.push(entry),
            },
            Err(message) => {
                let state = if candidate.projection.representation
                    == crate::manifest::PromptRepresentation::Symlink
                    && !crate::prompts::capability::symlink(
                        candidate.projection.agent,
                        candidate.projection.scope,
                    ) {
                    SyncState::Unsupported
                } else {
                    SyncState::Refused
                };
                refused.push(SyncEntry::new(candidate.id, state, message));
            }
        }
    }
    if !refused.is_empty() {
        refused.extend(prepared.into_iter().map(|(candidate, _)| {
            SyncEntry::new(
                candidate.id,
                SyncState::Refused,
                "selection failed validation; artifacts preserved",
            )
        }));
        return refused;
    }
    let entries = prepared
        .into_iter()
        .map(
            |(candidate, projection)| match selection::contents(roots, manifest, &candidate) {
                Ok(contents) if contents == projection.expected() => projection.publish(),
                _ => SyncEntry::new(
                    candidate.id,
                    SyncState::Failed,
                    "canonical source changed before publication",
                ),
            },
        )
        .collect();
    verify_doctor(roots, manifest, resource, agent, scope, entries)
}

fn verify_doctor(
    roots: &Roots,
    manifest: &Manifest,
    resource: SyncResource,
    agent: Agent,
    scope: Scope,
    entries: Vec<SyncEntry>,
) -> Vec<SyncEntry> {
    let diagnostics = if resource == SyncResource::Prompts {
        crate::prompts::diagnose(roots, manifest, Some(agent), Some(scope))
    } else {
        crate::commands::diagnose(roots, manifest, Some(agent), Some(scope))
    };
    let healthy = diagnostics
        .iter()
        .all(|diagnostic| diagnostic.state == State::Healthy);
    entries
        .into_iter()
        .map(|entry| {
            if healthy || !matches!(entry.state, SyncState::Applied | SyncState::Current) {
                entry
            } else {
                SyncEntry::new(
                    entry.id,
                    SyncState::Failed,
                    "Markdown publication did not pass Doctor verification",
                )
            }
        })
        .collect()
}

const fn supported(resource: SyncResource, agent: Agent, scope: Scope) -> bool {
    if matches!(resource, SyncResource::Commands) {
        matches!(
            (agent, scope),
            (Agent::Claude, Scope::User | Scope::Project)
        )
    } else if matches!(resource, SyncResource::Prompts) {
        matches!(
            (agent, scope),
            (Agent::Claude, Scope::User | Scope::Project) | (Agent::Cursor, Scope::Project)
        )
    } else {
        false
    }
}
