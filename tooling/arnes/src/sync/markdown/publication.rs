use super::{owned, selection};
use crate::Roots;
use crate::manifest::{Manifest, PromptRepresentation, Scope};
use crate::sync::{SyncEntry, SyncState, links, selection::LinkIntent, sources};

pub(super) enum Prepared {
    File(owned::Prepared),
    Link {
        prepared: links::PreparedLink,
        contents: String,
    },
}

impl Prepared {
    pub(super) fn expected(&self) -> &str {
        match self {
            Self::File(prepared) => prepared.expected(),
            Self::Link { contents, .. } => contents,
        }
    }

    pub(super) fn publish(self) -> SyncEntry {
        match self {
            Self::File(prepared) => prepared.publish(),
            Self::Link { prepared, .. } => links::publish(prepared),
        }
    }
}

pub(super) fn prepare(
    roots: &Roots,
    manifest: &Manifest,
    candidate: &selection::Candidate<'_>,
    contents: String,
) -> Result<Prepared, SyncEntry> {
    let refused = |message| SyncEntry::new(&candidate.id, SyncState::Refused, message);
    if candidate.projection.representation != PromptRepresentation::Symlink {
        let intent = selection::intent(roots, manifest, candidate, contents).map_err(refused)?;
        return owned::prepare(intent).map(Prepared::File);
    }
    let root = match candidate.projection.scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let destination = candidate.projection.destination;
    if std::fs::symlink_metadata(root.join(destination))
        .is_err_and(|error| error.kind() == std::io::ErrorKind::NotFound)
    {
        sources::protect_mutations(
            roots,
            manifest,
            candidate.projection.scope,
            &[destination.to_owned()],
        )
        .map_err(refused)?;
    }
    let intent = LinkIntent::regular_file(
        roots,
        candidate.id.clone(),
        root.to_owned(),
        destination.to_owned(),
        roots.repository().join(candidate.prompt.source()),
        roots.repository().to_owned(),
        candidate.projection.scope,
    )
    .map_err(|message| SyncEntry::new(&candidate.id, SyncState::Refused, message))?;
    links::prepare(intent).map(|prepared| Prepared::Link { prepared, contents })
}
