use super::{Failure, source::Expected};
use crate::Roots;
use crate::diagnostic::State;
use crate::files::paths::{ancestor_within, canonical_within, destination, label};
use crate::manifest::{Prompt, PromptProjection, PromptRepresentation, Scope};
use std::fs;
use std::path::Path;

pub fn validate(
    roots: &Roots,
    prompt: Prompt<'_>,
    projection: PromptProjection<'_>,
    expected: &Expected,
) -> Result<String, Failure> {
    let destination = destination(roots, projection.scope, projection.destination);
    let boundary = match projection.scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    if projection.representation == PromptRepresentation::Symlink {
        return read_symlink(roots, prompt, &destination, boundary, &expected.direct);
    }
    let actual = read_regular(&destination, boundary)?;
    let expected = match projection.representation {
        PromptRepresentation::File | PromptRepresentation::Symlink => &expected.direct,
        PromptRepresentation::Rendered => &expected.rendered,
    };
    if actual == *expected {
        Ok(actual)
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
    let metadata = fs::symlink_metadata(destination).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Failure::new(
                State::Drift,
                format!("destination {} is missing", destination.display()),
                "destination missing",
            )
        } else {
            Failure::new(
                State::Error,
                format!("destination {} could not be read", destination.display()),
                "destination unreadable",
            )
        }
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

const fn representation(representation: PromptRepresentation) -> &'static str {
    match representation {
        PromptRepresentation::File => "direct",
        PromptRepresentation::Rendered => "rendered",
        PromptRepresentation::Symlink => "symlink",
    }
}

fn read_symlink(
    roots: &Roots,
    prompt: Prompt<'_>,
    destination: &Path,
    boundary: &Path,
    expected: &str,
) -> Result<String, Failure> {
    if !ancestor_within(destination.parent().unwrap_or(boundary), boundary) {
        return Err(Failure::new(
            State::Error,
            "destination escapes its scope root",
            "destination escapes scope root",
        ));
    }
    let metadata = fs::symlink_metadata(destination).map_err(|error| {
        if error.kind() == std::io::ErrorKind::NotFound {
            Failure::new(
                State::Drift,
                "destination is missing",
                "destination missing",
            )
        } else {
            Failure::new(
                State::Error,
                "destination could not be inspected",
                "destination unreadable",
            )
        }
    })?;
    if !metadata.file_type().is_symlink() {
        return Err(Failure::new(
            State::Drift,
            "destination is not the expected symbolic link",
            "destination has wrong type",
        ));
    }
    let source = canonical_within(
        &roots.repository().join(prompt.source()),
        roots.repository(),
    )
    .ok_or_else(|| {
        Failure::new(
            State::Error,
            "source could not be confined",
            "invalid source",
        )
    })?;
    if fs::canonicalize(destination).ok().as_ref() != Some(&source) {
        return Err(Failure::new(
            State::Drift,
            "destination is a divergent or dangling link",
            "destination has wrong link",
        ));
    }
    let actual = fs::read_to_string(destination).map_err(|_| {
        Failure::new(
            State::Error,
            "destination could not be read",
            "destination unreadable",
        )
    })?;
    if actual != expected {
        return Err(Failure::new(
            State::Drift,
            "linked source changed during diagnosis",
            "projection stale",
        ));
    }
    Ok(actual)
}
