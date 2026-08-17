use super::Failure;
use crate::Roots;
use crate::diagnostic::State;
use crate::files::paths::{canonical_within, destination, label, planned_within};
use crate::manifest::{Manifest, Prompt, PromptProjection, PromptRepresentation, Scope};
use std::collections::HashMap;
use std::path::PathBuf;

pub struct Tracker {
    sources: HashMap<PathBuf, String>,
    destinations: HashMap<PathBuf, DestinationOwner>,
}

impl Tracker {
    pub fn new(roots: &Roots, manifest: &Manifest) -> Self {
        let mut tracker = Self {
            sources: HashMap::new(),
            destinations: HashMap::new(),
        };
        for (scope, path) in manifest.resource_destinations() {
            let destination = destination(roots, scope, path);
            if let Some(identity) = planned_within(&destination, boundary(roots, scope)) {
                tracker
                    .destinations
                    .insert(identity, DestinationOwner::resource(label(scope, path)));
            }
        }
        tracker
    }

    pub fn validate(
        &mut self,
        roots: &Roots,
        prompt: Prompt<'_>,
        projection: PromptProjection<'_>,
    ) -> Result<(), Failure> {
        if projection.representation == PromptRepresentation::Symlink {
            return Ok(());
        }
        let source = roots.repository().join(prompt.source());
        if let Some(identity) = canonical_within(&source, roots.repository()) {
            if let Some(previous) = self.destinations.get(&identity)
                && previous.prompt.as_deref() != Some(prompt.id())
            {
                return Err(ambiguous("source", prompt.id(), &previous.label));
            }
            if let Some(previous) = self.sources.get(&identity) {
                if previous != prompt.id() {
                    return Err(ambiguous("source", prompt.id(), previous));
                }
            } else {
                self.sources.insert(identity, prompt.id().to_owned());
            }
        }

        let destination = destination(roots, projection.scope, projection.destination);
        if let Some(identity) = planned_within(&destination, boundary(roots, projection.scope)) {
            let current = label(projection.scope, projection.destination);
            if let Some(previous) = self.sources.get(&identity) {
                let direct = projection.scope == Scope::Project
                    && projection.representation == PromptRepresentation::File
                    && previous == prompt.id();
                if !direct {
                    return Err(ambiguous("destination", &current, previous));
                }
            }
            let owner = DestinationOwner::prompt(prompt.id(), current.clone());
            if let Some(previous) = self.destinations.insert(identity, owner) {
                return Err(ambiguous("destination", &current, &previous.label));
            }
        }
        Ok(())
    }
}

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

    fn resource(label: String) -> Self {
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
