use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::includes::IncludeError;
use crate::files::paths::{parent_within, resolves_within, same_file};
use crate::manifest::{InstructionResource, Scope};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn expected_link(
    destination: &Path,
    source: &Path,
    root: &Path,
    subject: &str,
) -> Result<(), Diagnostic> {
    if !parent_within(destination, root) {
        return Err(outside_destination(subject, destination));
    }
    let metadata = fs::symlink_metadata(destination).map_err(|error| match error.kind() {
        ErrorKind::NotFound => destination_error(subject, destination, State::Drift, "is missing"),
        _ => destination_error(subject, destination, State::Error, "could not be read"),
    })?;
    if !metadata.file_type().is_symlink() {
        return Err(wrong_link(subject, destination));
    }
    if !same_file(destination, source) {
        return Err(wrong_link(subject, destination));
    }
    Ok(())
}

pub fn expected_file(destination: &Path, root: &Path, subject: &str) -> Result<(), Diagnostic> {
    let metadata = fs::symlink_metadata(destination).map_err(|error| match error.kind() {
        ErrorKind::NotFound => destination_error(subject, destination, State::Drift, "is missing"),
        _ => destination_error(subject, destination, State::Error, "could not be read"),
    })?;
    if !metadata.file_type().is_file() {
        return Err(destination_error(
            subject,
            destination,
            State::Drift,
            "is not a regular file",
        ));
    }
    if !resolves_within(destination, root) {
        return Err(outside_destination(subject, destination));
    }
    Ok(())
}

pub fn include_diagnostic(subject: &str, error: IncludeError, default: State) -> Diagnostic {
    let (state, message) = match error {
        IncludeError::Missing(path) => (default, format!("include {} is missing", path.display())),
        IncludeError::MissingLink(path) => (
            State::Drift,
            format!("include {} is missing", path.display()),
        ),
        IncludeError::Dangling(path) => (
            State::Error,
            format!("include {} is missing (dangling symlink)", path.display()),
        ),
        IncludeError::WrongLink(path) => (
            State::Drift,
            format!("include {} has the wrong symlink target", path.display()),
        ),
        IncludeError::NotFile(path) => (State::Error, format!("{} is not a file", path.display())),
        IncludeError::Unreadable(path) => (
            State::Error,
            format!("{} could not be read", path.display()),
        ),
        IncludeError::Escapes(path) => (
            State::Error,
            format!("include {path} escapes its instruction root"),
        ),
        IncludeError::OutsideRoot(path) => (
            State::Error,
            format!("{} resolves outside its instruction root", path.display()),
        ),
        IncludeError::Cycle(path) => (
            State::Error,
            format!("include cycle reaches {}", path.display()),
        ),
    };
    Diagnostic::new("instructions", state, format!("{subject}: {message}"))
}

pub fn source_diagnostic(subject: &str, error: IncludeError) -> Diagnostic {
    let message = match error {
        IncludeError::Missing(path) | IncludeError::MissingLink(path) => {
            format!("source {} is missing", path.display())
        }
        IncludeError::Dangling(path) => {
            format!("source {} is missing (dangling symlink)", path.display())
        }
        IncludeError::NotFile(path) => format!("source {} is not a file", path.display()),
        IncludeError::Unreadable(path) => format!("source {} could not be read", path.display()),
        IncludeError::WrongLink(path) => {
            format!("source {} has an unsupported symlink", path.display())
        }
        IncludeError::Escapes(path) => format!("source include {path} escapes the repository"),
        IncludeError::OutsideRoot(path) => {
            format!("source {} resolves outside the repository", path.display())
        }
        IncludeError::Cycle(path) => {
            format!("source include cycle reaches {}", path.display())
        }
    };
    Diagnostic::new(
        "instructions",
        State::Error,
        format!("{subject}: {message}"),
    )
}

pub fn destination(roots: &Roots, resource: &InstructionResource<'_>) -> PathBuf {
    match resource.scope {
        Scope::User => roots.home().join(resource.destination),
        Scope::Project => roots.repository().join(resource.destination),
    }
}

pub fn destination_label(resource: &InstructionResource<'_>) -> String {
    match resource.scope {
        Scope::User => format!("~/{}", resource.destination.display()),
        Scope::Project => resource.destination.display().to_string(),
    }
}

pub fn relative(path: &Path, roots: &Roots) -> String {
    path.strip_prefix(roots.home())
        .map(|path| format!("~/{}", path.display()))
        .or_else(|_| {
            path.strip_prefix(roots.repository())
                .map(|path| path.display().to_string())
        })
        .unwrap_or_else(|_| path.display().to_string())
}

pub fn healthy(subject: &str, destination: String) -> Diagnostic {
    Diagnostic::new(
        "instructions",
        State::Healthy,
        format!("{subject} destination {destination} is current"),
    )
}

fn wrong_link(subject: &str, destination: &Path) -> Diagnostic {
    destination_error(
        subject,
        destination,
        State::Drift,
        "has the wrong symlink target",
    )
}

fn outside_destination(subject: &str, destination: &Path) -> Diagnostic {
    destination_error(
        subject,
        destination,
        State::Error,
        "resolves outside its declared root",
    )
}

fn destination_error(subject: &str, path: &Path, state: State, reason: &str) -> Diagnostic {
    Diagnostic::new(
        "instructions",
        state,
        format!("{subject} destination {} {reason}", path.display()),
    )
}
