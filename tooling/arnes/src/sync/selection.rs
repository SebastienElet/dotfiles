use super::{SyncEntry, SyncResource, SyncState};
use crate::Roots;
use crate::files::paths::{canonical_within, planned_within};
use crate::manifest::{Agent, Manifest, Scope, SkillLayout};
use crate::skills::{discovery, references};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

pub(super) enum SourceKind {
    Rule,
    Skill,
    SkillRoot,
}

pub(super) struct LinkIntent {
    pub id: String,
    pub root: PathBuf,
    pub destination: PathBuf,
    source: PathBuf,
    source_root: PathBuf,
    kind: SourceKind,
}

impl LinkIntent {
    pub(super) fn regular_file(
        roots: &Roots,
        id: String,
        root: PathBuf,
        destination: PathBuf,
        source: PathBuf,
        source_root: PathBuf,
        scope: Scope,
    ) -> Result<Self, String> {
        let intent = Self {
            id,
            root,
            destination,
            source,
            source_root,
            kind: SourceKind::Rule,
        };
        if scope == Scope::User {
            validate_user_destination(roots, &intent)?;
        }
        Ok(intent)
    }

    pub fn protected_source(&self) -> Result<PathBuf, String> {
        let protected = match self.kind {
            SourceKind::Rule | SourceKind::SkillRoot => self.source.as_path(),
            SourceKind::Skill => self
                .source
                .parent()
                .ok_or_else(|| "skill collection has no parent".to_owned())?,
        };
        canonical_within(protected, &self.source_root)
            .ok_or_else(|| "canonical source could not be resolved".to_owned())
    }

    pub fn source(&self) -> Result<PathBuf, String> {
        let source = canonical_within(&self.source, &self.source_root)
            .ok_or_else(|| "source is missing or resolves outside its repository".to_owned())?;
        let metadata = fs::metadata(&source).map_err(|_| "source could not be read".to_owned())?;
        match self.kind {
            SourceKind::Rule => {
                if !metadata.is_file() || metadata.nlink() != 1 {
                    return Err("source must be a single-link regular file".to_owned());
                }
                fs::read_to_string(&source)
                    .map_err(|_| "source could not be read as text".to_owned())?;
            }
            SourceKind::Skill => {
                let collection = self.protected_source()?;
                validate_skill_collection(&collection, &self.source_root, Scope::User)?;
                if !source.starts_with(&collection) {
                    return Err("user skill resolves outside its canonical collection".to_owned());
                }
                validate_skill(&source, &self.id)?;
            }
            SourceKind::SkillRoot => {
                validate_skill_collection(&source, &self.source_root, Scope::Project)?;
                let names = discovery::installations(&source)?;
                if names.is_empty() {
                    return Err("source has no project skills".to_owned());
                }
                for name in names {
                    let skill = source.join(name);
                    let canonical = canonical_within(&skill, &source).ok_or_else(|| {
                        "project skill resolves outside its canonical collection".to_owned()
                    })?;
                    validate_skill(&canonical, &self.id)?;
                }
            }
        }
        Ok(source)
    }
}

fn canonical_source_boundary(path: &Path) -> Result<Option<PathBuf>, String> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(_) => Err("canonical source boundary could not be inspected".to_owned()),
        Ok(_) => fs::canonicalize(path)
            .map(Some)
            .map_err(|_| "canonical source boundary could not be resolved".to_owned()),
    }
}

fn validate_skill_collection(
    collection: &Path,
    repository: &Path,
    scope: Scope,
) -> Result<(), String> {
    let other = match scope {
        Scope::User => ".agents/skills",
        Scope::Project => "harness/skills",
    };
    if let Some(other) = canonical_source_boundary(&repository.join(other))?
        && (collection.starts_with(&other) || other.starts_with(collection))
    {
        return Err(format!(
            "{scope} skill collection overlaps the other canonical scope"
        ));
    }
    Ok(())
}

fn validate_user_destination(roots: &Roots, intent: &LinkIntent) -> Result<(), String> {
    let destination = intent.root.join(&intent.destination);
    let parent = destination
        .parent()
        .and_then(|parent| planned_within(parent, &intent.root))
        .ok_or_else(|| "destination parent could not be confined to its user scope".to_owned())?;
    let name = destination
        .file_name()
        .ok_or_else(|| "destination has no filename".to_owned())?;
    let planned = parent.join(name);
    for repository in [roots.repository(), roots.deployment_repository()] {
        for boundary in ["home", "harness", "harness/skills", ".agents/skills"] {
            if let Some(source) = canonical_source_boundary(&repository.join(boundary))?
                && planned.starts_with(source)
            {
                return Err("user destination lies inside canonical repository sources".to_owned());
            }
        }
    }
    Ok(())
}

fn validate_skill(source: &Path, subject: &str) -> Result<(), String> {
    if !fs::metadata(source).is_ok_and(|metadata| metadata.is_dir()) {
        return Err("skill source must be a directory".to_owned());
    }
    let metadata = fs::metadata(source.join("SKILL.md"))
        .map_err(|_| "SKILL.md is missing or unreadable".to_owned())?;
    if !metadata.is_file() || metadata.nlink() != 1 {
        return Err("SKILL.md must be a single-link regular file".to_owned());
    }
    references::validate(source, subject).map_err(|diagnostic| diagnostic.message)
}

fn select_rules(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Result<Vec<LinkIntent>, SyncEntry> {
    if scope != Scope::User || !matches!(agent, Agent::Claude | Agent::Cursor) {
        return Err(SyncEntry::new(
            "selection",
            SyncState::Unsupported,
            "rules support Claude and Cursor user only",
        ));
    }
    Ok(manifest
        .rule_resources()
        .filter(|rule| rule.agent == agent && rule.scope == scope)
        .map(|rule| LinkIntent {
            id: rule.id.to_owned(),
            root: roots.home().to_owned(),
            destination: rule.destination.to_owned(),
            source: roots.repository().join(rule.source),
            source_root: roots.repository().to_owned(),
            kind: SourceKind::Rule,
        })
        .collect())
}

pub(super) fn select(
    roots: &Roots,
    manifest: &Manifest,
    resource: SyncResource,
    agent: Agent,
    scope: Scope,
) -> Result<Vec<LinkIntent>, SyncEntry> {
    if !manifest.combinations().any(|pair| pair == (agent, scope)) {
        return Err(SyncEntry::new(
            "selection",
            SyncState::Unsupported,
            "agent and scope are not declared",
        ));
    }
    let root = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let mut selected = Vec::new();
    match resource {
        SyncResource::Rules => {
            selected = select_rules(roots, manifest, agent, scope)?;
        }
        SyncResource::Skills => {
            for projection in manifest
                .skill_projections()
                .filter(|projection| projection.agent == agent && projection.scope == scope)
            {
                let source_root = match scope {
                    Scope::User => roots.deployment_repository(),
                    Scope::Project => roots.repository(),
                };
                match (scope, projection.layout, projection.source) {
                    (Scope::User, SkillLayout::Leaves, source)
                        if source == Path::new("harness/skills") =>
                    {
                        for slug in manifest.installed_skills(agent, scope) {
                            selected.push(LinkIntent {
                                id: format!("{}:{slug}", projection.id),
                                root: root.to_owned(),
                                destination: projection.destination.join(slug),
                                source: source_root.join(source).join(slug),
                                source_root: source_root.to_owned(),
                                kind: SourceKind::Skill,
                            });
                        }
                    }
                    (Scope::Project, SkillLayout::Root, source)
                        if source == Path::new(".agents/skills") =>
                    {
                        selected.push(LinkIntent {
                            id: projection.id.to_owned(),
                            root: root.to_owned(),
                            destination: projection.destination.to_owned(),
                            source: source_root.join(source),
                            source_root: source_root.to_owned(),
                            kind: SourceKind::SkillRoot,
                        });
                    }
                    _ => {
                        return Err(SyncEntry::new(
                            projection.id,
                            SyncState::Unsupported,
                            "skill layout or source is not supported",
                        ));
                    }
                }
            }
        }
        SyncResource::Config
        | SyncResource::Instructions
        | SyncResource::Prompts
        | SyncResource::Commands
        | SyncResource::Mcp
        | SyncResource::Statusline => {
            return Err(SyncEntry::new(
                "selection",
                SyncState::Unsupported,
                "resource is not a link projection",
            ));
        }
    }
    if selected.is_empty() {
        return Err(SyncEntry::new(
            "selection",
            SyncState::Empty,
            "no managed projections selected",
        ));
    }
    if scope == Scope::User {
        for intent in &selected {
            validate_user_destination(roots, intent)
                .map_err(|message| SyncEntry::new(&intent.id, SyncState::Refused, message))?;
        }
    }
    Ok(selected)
}
