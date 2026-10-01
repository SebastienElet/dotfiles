use super::Failure;
use crate::Roots;
use crate::diagnostic::State;
use crate::files::paths::{
    FileIdentity, canonical_within, destination, file_identity_within, label, planned_within,
};
use crate::manifest::{Manifest, Prompt, PromptProjection, PromptRepresentation, Scope};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

pub struct Tracker {
    sources: HashMap<Identity, Vec<String>>,
    destinations: HashMap<Identity, DestinationOwner>,
}

impl Tracker {
    pub fn new_for_scopes(roots: &Roots, manifest: &Manifest, scopes: &[Scope]) -> Self {
        let mut tracker = Self {
            sources: HashMap::new(),
            destinations: HashMap::new(),
        };
        for (scope, path) in manifest
            .resource_destinations()
            .filter(|(scope, _)| scopes.contains(scope))
        {
            tracker.seed_resource(roots, scope, path);
        }
        for prompt in manifest.prompts().filter(|prompt| {
            prompt.projections().any(|projection| {
                scopes.contains(&projection.scope) && !unsupported_link(projection)
            })
        }) {
            let source = roots.repository().join(prompt.source());
            for identity in canonical_identities(&source, roots.repository()) {
                tracker.record_source(identity, prompt);
            }
        }
        tracker
    }

    pub fn relevant_scopes(roots: &Roots, selected: &[Scope]) -> Vec<Scope> {
        let mut scopes = selected.iter().copied().collect::<HashSet<_>>();
        let shared = matches!(
            (
                fs::canonicalize(roots.home()).ok(),
                fs::canonicalize(roots.repository()).ok()
            ),
            (Some(home), Some(repository)) if home == repository
        );
        if shared {
            scopes.extend([Scope::User, Scope::Project]);
        }
        scopes.into_iter().collect()
    }

    pub fn seed_projection_destination(
        &mut self,
        roots: &Roots,
        prompt: Prompt<'_>,
        projection: PromptProjection<'_>,
    ) {
        if unsupported_link(projection) {
            return;
        }
        let destination = destination(roots, projection.scope, projection.destination);
        let owner =
            DestinationOwner::prompt(prompt.id(), label(projection.scope, projection.destination));
        for identity in destination_identities(
            &destination,
            boundary(roots, projection.scope),
            projection.representation,
        ) {
            self.destinations
                .entry(identity)
                .or_insert_with(|| owner.clone());
        }
    }

    pub fn validate(
        &mut self,
        roots: &Roots,
        prompt: Prompt<'_>,
        projection: PromptProjection<'_>,
    ) -> Result<(), Failure> {
        if unsupported_link(projection) {
            return Ok(());
        }
        self.validate_source(roots, prompt)?;
        self.validate_destination(roots, prompt, projection)
    }

    fn seed_resource(&mut self, roots: &Roots, scope: Scope, path: &std::path::Path) {
        let destination = destination(roots, scope, path);
        let owner = DestinationOwner::resource(&label(scope, path));
        for identity in planned_identities(&destination, boundary(roots, scope)) {
            self.destinations
                .entry(identity)
                .or_insert_with(|| owner.clone());
        }
    }

    fn validate_source(&mut self, roots: &Roots, prompt: Prompt<'_>) -> Result<(), Failure> {
        let source = roots.repository().join(prompt.source());
        let identities = canonical_identities(&source, roots.repository());
        for identity in &identities {
            if let Some(previous) = self.destinations.get(identity)
                && previous.prompt.as_deref() != Some(prompt.id())
            {
                return Err(ambiguous("source", prompt.id(), &previous.label));
            }
            if let Some(previous) = self
                .sources
                .get(identity)
                .and_then(|owners| owners.iter().find(|owner| owner.as_str() != prompt.id()))
            {
                return Err(ambiguous("source", prompt.id(), previous));
            }
        }
        for identity in identities {
            self.record_source(identity, prompt);
        }
        Ok(())
    }

    fn record_source(&mut self, identity: Identity, prompt: Prompt<'_>) {
        let owners = self.sources.entry(identity).or_default();
        if !owners.iter().any(|owner| owner == prompt.id()) {
            owners.push(prompt.id().to_owned());
        }
    }

    fn validate_destination(
        &mut self,
        roots: &Roots,
        prompt: Prompt<'_>,
        projection: PromptProjection<'_>,
    ) -> Result<(), Failure> {
        let destination = destination(roots, projection.scope, projection.destination);
        let identities = destination_identities(
            &destination,
            boundary(roots, projection.scope),
            projection.representation,
        );
        let current = label(projection.scope, projection.destination);
        for identity in &identities {
            if let Some(previous) = self.sources.get(identity).and_then(|owners| {
                owners.iter().find(|owner| {
                    projection.scope != Scope::Project
                        || projection.representation != PromptRepresentation::File
                        || owner.as_str() != prompt.id()
                })
            }) {
                return Err(ambiguous("destination", &current, previous));
            }
            if let Some(previous) = self.destinations.get(identity) {
                return Err(ambiguous("destination", &current, &previous.label));
            }
        }
        let owner = DestinationOwner::prompt(prompt.id(), current);
        for identity in identities {
            self.destinations.insert(identity, owner.clone());
        }
        Ok(())
    }
}

#[derive(Eq, Hash, PartialEq)]
enum Identity {
    Path(PathBuf),
    File(FileIdentity),
}

fn canonical_identities(path: &std::path::Path, root: &std::path::Path) -> Vec<Identity> {
    let Some(canonical) = canonical_within(path, root) else {
        return Vec::new();
    };
    let mut identities = vec![Identity::Path(canonical)];
    if let Some(identity) = file_identity_within(path, root) {
        identities.push(Identity::File(identity));
    }
    identities
}

fn planned_identities(path: &std::path::Path, root: &std::path::Path) -> Vec<Identity> {
    let Some(planned) = planned_within(path, root) else {
        return Vec::new();
    };
    let mut identities = vec![Identity::Path(planned)];
    if let Some(identity) = file_identity_within(path, root) {
        identities.push(Identity::File(identity));
    }
    identities
}

#[derive(Clone)]
struct DestinationOwner {
    prompt: Option<String>,
    label: String,
}

impl DestinationOwner {
    fn prompt(prompt: &str, label: String) -> Self {
        Self {
            prompt: Some(prompt.to_owned()),
            label,
        }
    }

    fn resource(label: &str) -> Self {
        Self {
            prompt: None,
            label: format!("resource {label}"),
        }
    }
}

fn boundary(roots: &Roots, scope: Scope) -> &std::path::Path {
    match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    }
}

fn ambiguous(kind: &str, current: &str, previous: &str) -> Failure {
    Failure::new(
        State::Error,
        format!("{kind} {current} aliases managed {kind} {previous}"),
        format!("ambiguous {kind}"),
    )
}

fn destination_identities(
    path: &std::path::Path,
    root: &std::path::Path,
    representation: PromptRepresentation,
) -> Vec<Identity> {
    if representation != PromptRepresentation::Symlink {
        return planned_identities(path, root);
    }
    let Some(parent) = path
        .parent()
        .and_then(|parent| planned_within(parent, root))
    else {
        return Vec::new();
    };
    let Some(name) = path.file_name() else {
        return Vec::new();
    };
    let mut identities = vec![Identity::Path(parent.join(name))];
    if fs::symlink_metadata(path).is_ok_and(|metadata| !metadata.file_type().is_symlink())
        && let Some(identity) = file_identity_within(path, root)
    {
        identities.push(Identity::File(identity));
    }
    identities
}

fn unsupported_link(projection: PromptProjection<'_>) -> bool {
    projection.representation == PromptRepresentation::Symlink
        && !super::capability::symlink(projection.agent, projection.scope)
}
