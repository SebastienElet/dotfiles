use super::markdown::owned::{self, Intent, Prepared};
use super::selection::LinkIntent;
use super::{SyncEntry, SyncState};
use crate::Roots;
use crate::diagnostic::State;
use crate::files::includes::{self, Resolver};
use crate::instructions::projection::{self, Projection};
use crate::manifest::{Agent, InstructionResource, Manifest, Scope};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Component, Path, PathBuf};

pub(super) fn synchronize(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Vec<SyncEntry> {
    let Some(kind) = projection::kind(agent, scope) else {
        return vec![SyncEntry::new(
            "selection",
            SyncState::Unsupported,
            "instruction combination is unsupported",
        )];
    };
    if !manifest.combinations().any(|pair| pair == (agent, scope)) {
        return vec![SyncEntry::new(
            "selection",
            SyncState::Unsupported,
            "instruction combination is not declared",
        )];
    }
    let selected = manifest
        .instruction_resources()
        .filter(|resource| resource.agent == agent && resource.scope == scope)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return vec![SyncEntry::new(
            "selection",
            SyncState::Empty,
            "no managed instruction projections selected",
        )];
    }
    let entries = if matches!(kind, Projection::Link) {
        publish_links(roots, manifest, &selected)
    } else {
        publish_files(roots, manifest, &selected, kind)
    };
    verify_doctor(roots, manifest, agent, scope, entries)
}

fn publish_links(
    roots: &Roots,
    manifest: &Manifest,
    selected: &[InstructionResource<'_>],
) -> Vec<SyncEntry> {
    let mut intents = Vec::new();
    for resource in selected {
        let source = roots.deployment_repository().join(resource.source);
        if Resolver::new(roots.deployment_repository())
            .walk(&source)
            .is_err()
        {
            return refused_selection(selected, "instruction source or include graph is invalid");
        }
        match LinkIntent::regular_file(
            roots,
            resource.id.to_owned(),
            roots.home().to_owned(),
            resource.destination.to_owned(),
            source,
            roots.deployment_repository().to_owned(),
            Scope::User,
        ) {
            Ok(intent) => intents.push(intent),
            Err(_) => {
                return refused_selection(
                    selected,
                    "instruction destination aliases canonical sources",
                );
            }
        }
    }
    super::synchronize_links(roots, manifest, Scope::User, intents)
}

fn publish_files(
    roots: &Roots,
    manifest: &Manifest,
    selected: &[InstructionResource<'_>],
    kind: Projection,
) -> Vec<SyncEntry> {
    let mut prepared: Vec<(InstructionResource<'_>, Prepared)> = Vec::new();
    for resource in selected {
        let contents = match expected(roots, resource, kind) {
            Ok(contents) => contents,
            Err(message) => return refused_selection(selected, message),
        };
        let intent = match intent(roots, manifest, resource, contents) {
            Ok(intent) => intent,
            Err(message) => return refused_selection(selected, message),
        };
        match owned::prepare(intent) {
            Ok(projection) => prepared.push((*resource, projection)),
            Err(entry) => return refused_selection(selected, &entry.message),
        }
    }
    prepared
        .into_iter()
        .map(
            |(resource, prepared)| match expected(roots, &resource, kind) {
                Ok(contents) if contents == prepared.expected() => prepared.publish(),
                _ => SyncEntry::new(
                    resource.id,
                    SyncState::Failed,
                    "instruction source changed before publication",
                ),
            },
        )
        .collect()
}

fn expected(
    roots: &Roots,
    resource: &InstructionResource<'_>,
    kind: Projection,
) -> Result<String, &'static str> {
    let source_root = source_root(roots, resource.scope);
    let source = source_root.join(resource.source);
    let resolver = Resolver::new(source_root);
    let contents = resolver
        .read(&source)
        .map_err(|_| "instruction source is unreadable")?;
    resolver
        .walk(&source)
        .map_err(|_| "instruction include graph is invalid")?;
    match kind {
        Projection::Generated => projection::generated_contents(source_root, &source, &contents)
            .map_err(|_| "instruction assembly is invalid"),
        Projection::Include => {
            owned::validate_parent(roots.repository(), resource.destination)?;
            let destination = roots.repository().join(resource.destination);
            if fs::symlink_metadata(&destination)
                .is_ok_and(|metadata| metadata.is_file() && metadata.nlink() == 1)
                && resolver
                    .walk(&destination)
                    .is_ok_and(|graph| graph.contains(&source))
            {
                return resolver
                    .read(&destination)
                    .map_err(|_| "project instructions could not be read");
            }
            include_line(resource.source, resource.destination)
        }
        Projection::Link => Err("link instructions do not render content"),
    }
}

fn include_line(source: &Path, destination: &Path) -> Result<String, &'static str> {
    let parent = destination
        .parent()
        .ok_or("instruction destination has no parent")?;
    let source = source.components().collect::<Vec<_>>();
    let parent = parent.components().collect::<Vec<_>>();
    let common = source
        .iter()
        .zip(&parent)
        .take_while(|(left, right)| left == right)
        .count();
    let mut relative = PathBuf::new();
    for _ in parent.iter().skip(common) {
        relative.push("..");
    }
    for component in source.iter().skip(common) {
        if !matches!(component, Component::Normal(_)) {
            return Err("instruction source path is invalid");
        }
        relative.push(component.as_os_str());
    }
    let name = relative
        .to_str()
        .ok_or("instruction include path is invalid")?;
    let contents = format!("@{name}\n");
    if includes::leading_imports(&contents) != [name] {
        return Err("instruction include cannot be represented by the agent syntax");
    }
    Ok(contents)
}

fn intent(
    roots: &Roots,
    manifest: &Manifest,
    resource: &InstructionResource<'_>,
    contents: String,
) -> Result<Intent, &'static str> {
    let root = match resource.scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let source_root = source_root(roots, resource.scope);
    if resource.scope == Scope::User {
        LinkIntent::regular_file(
            roots,
            resource.id.to_owned(),
            root.to_owned(),
            resource.destination.to_owned(),
            source_root.join(resource.source),
            source_root.to_owned(),
            Scope::User,
        )
        .map_err(|_| "user instruction destination aliases canonical sources")?;
    }
    if !fs::read(root.join(resource.destination)).is_ok_and(|bytes| bytes == contents.as_bytes()) {
        let name = resource
            .destination
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("publication filename is invalid")?;
        let receipt = resource
            .destination
            .with_file_name(format!(".{name}.arnes.json"));
        let paths = super::sources::publication_paths(&[resource.destination.to_owned(), receipt])?;
        super::sources::protect_mutations(roots, manifest, resource.scope, &paths)?;
    }
    Ok(Intent {
        id: resource.id.to_owned(),
        owner: format!("instructions:{}", resource.id),
        root: root.to_owned(),
        destination: resource.destination.to_owned(),
        contents,
    })
}

fn source_root(roots: &Roots, scope: Scope) -> &Path {
    match scope {
        Scope::User => roots.deployment_repository(),
        Scope::Project => roots.repository(),
    }
}

fn refused_selection(selected: &[InstructionResource<'_>], message: &str) -> Vec<SyncEntry> {
    selected
        .iter()
        .map(|resource| SyncEntry::new(resource.id, SyncState::Refused, message))
        .collect()
}

fn verify_doctor(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
    entries: Vec<SyncEntry>,
) -> Vec<SyncEntry> {
    let healthy = crate::instructions::diagnose(roots, manifest, Some(agent), Some(scope))
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
                    "instruction publication did not pass Doctor verification",
                )
            }
        })
        .collect()
}
