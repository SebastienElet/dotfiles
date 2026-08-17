use super::{Failure, source::Expected};
use crate::Roots;
use crate::diagnostic::State;
use crate::files::paths::{ancestor_within, canonical_within, destination, label};
use crate::manifest::{PromptProjection, PromptRepresentation, Scope};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn validate(
    roots: &Roots,
    projection: PromptProjection<'_>,
    expected: &Expected,
) -> Result<(), Failure> {
    let destination = destination(roots, projection.scope, projection.destination);
    let boundary = match projection.scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let actual = read_regular(&destination, boundary)?;
    let expected = match projection.representation {
        PromptRepresentation::File => &expected.direct,
        PromptRepresentation::Rendered => &expected.rendered,
        PromptRepresentation::Symlink => unreachable!(),
    };
    if actual == *expected {
        Ok(())
    } else {
        Err(stale(projection))
    }
}

fn read_regular(destination: &Path, boundary: &Path) -> Result<String, Failure> {
    if !ancestor_within(destination.parent().unwrap_or(boundary), boundary) {
        return Err(Failure::new(
            State::Error,
            format!(
                "destination {} escapes its scope root",
                destination.display()
            ),
            "destination escapes scope root",
        ));
    }
    let metadata = fs::symlink_metadata(destination).map_err(|error| match error.kind() {
        ErrorKind::NotFound => Failure::new(
            State::Drift,
            format!("destination {} is missing", destination.display()),
            "destination missing",
        ),
        _ => Failure::new(
            State::Error,
            format!("destination {} could not be read", destination.display()),
            "destination unreadable",
        ),
    })?;
    if metadata.file_type().is_symlink() {
        return Err(Failure::new(
            State::Drift,
            format!(
                "destination {} is a symlink instead of the expected regular file",
                destination.display()
            ),
            "destination has wrong link",
        ));
    }
    if !metadata.file_type().is_file() {
        return Err(Failure::new(
            State::Drift,
            format!(
                "destination {} is not a regular file",
                destination.display()
            ),
            "destination has wrong type",
        ));
    }
    if canonical_within(destination, boundary).is_none() {
        return Err(Failure::new(
            State::Error,
            format!(
                "destination {} escapes its scope root",
                destination.display()
            ),
            "destination escapes scope root",
        ));
    }
    fs::read_to_string(destination).map_err(|_| {
        Failure::new(
            State::Error,
            format!("destination {} could not be read", destination.display()),
            "destination unreadable",
        )
    })
}

fn stale(projection: PromptProjection<'_>) -> Failure {
    Failure::new(
        State::Drift,
        format!(
            "{} file {} is stale",
            representation(projection.representation),
            label(projection.scope, projection.destination)
        ),
        "projection stale",
    )
}

fn representation(representation: PromptRepresentation) -> &'static str {
    match representation {
        PromptRepresentation::File => "direct",
        PromptRepresentation::Rendered => "rendered",
        PromptRepresentation::Symlink => unreachable!(),
    }
}
