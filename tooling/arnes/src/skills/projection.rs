use super::references;
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within, destination, label};
use crate::manifest::{Scope, SkillProjection};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub fn leaf(roots: &Roots, resource: &SkillProjection<'_>, name: &str) -> Diagnostic {
    let installed = resource.destination.join(name);
    let subject = format!(
        "managed {} {} skill {name} at {}",
        resource.agent,
        resource.scope,
        label(resource.scope, &installed)
    );
    let source = roots.repository().join(resource.source).join(name);
    let destination = destination(roots, resource.scope, &installed);
    match expected_link(roots, resource.scope, &source, &destination, &subject)
        .and_then(|_| references::validate(&destination, &subject))
    {
        Ok(()) => Diagnostic::new("skills", State::Healthy, format!("{subject} is current")),
        Err(diagnostic) => diagnostic,
    }
}

pub fn root(
    roots: &Roots,
    resource: &SkillProjection<'_>,
    skills: &[&str],
) -> Result<Vec<Diagnostic>, Diagnostic> {
    let source = roots.repository().join(resource.source);
    let destination = destination(roots, resource.scope, resource.destination);
    let subject = format!(
        "managed {} {} skills projection at {}",
        resource.agent,
        resource.scope,
        label(resource.scope, resource.destination)
    );
    expected_link(roots, resource.scope, &source, &destination, &subject)?;
    Ok(skills
        .iter()
        .map(|name| root_skill(roots, resource, &source, &destination, Path::new(name)))
        .collect())
}

fn root_skill(
    roots: &Roots,
    resource: &SkillProjection<'_>,
    source: &Path,
    destination: &Path,
    name: &Path,
) -> Diagnostic {
    let source = source.join(name);
    let installed = destination.join(name);
    let subject = format!(
        "managed {} {} skill {} at {}/{}",
        resource.agent,
        resource.scope,
        name.display(),
        label(resource.scope, resource.destination),
        name.display()
    );
    let result = source_directory(&source, roots.repository(), &subject)
        .and_then(|expected| same_target(&installed, &expected, &subject))
        .and_then(|_| references::validate(&installed, &subject));
    match result {
        Ok(()) => Diagnostic::new("skills", State::Healthy, format!("{subject} is current")),
        Err(diagnostic) => diagnostic,
    }
}

fn expected_link(
    roots: &Roots,
    scope: Scope,
    source: &Path,
    destination: &Path,
    subject: &str,
) -> Result<(), Diagnostic> {
    let expected = source_directory(source, roots.repository(), subject)?;
    let boundary = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    if !ancestor_within(destination.parent().unwrap_or(boundary), boundary) {
        return Err(broken(
            subject,
            State::Error,
            format!(
                "destination {} escapes its scope root",
                destination.display()
            ),
        ));
    }
    let metadata = fs::symlink_metadata(destination).map_err(|error| match error.kind() {
        ErrorKind::NotFound => broken(
            subject,
            State::Drift,
            format!("destination {} is missing", destination.display()),
        ),
        _ => broken(
            subject,
            State::Error,
            format!("destination {} could not be read", destination.display()),
        ),
    })?;
    if !metadata.file_type().is_symlink() {
        return Err(broken(
            subject,
            State::Drift,
            format!("destination {} is not a symlink", destination.display()),
        ));
    }
    same_target(destination, &expected, subject)
}

fn source_directory(source: &Path, root: &Path, subject: &str) -> Result<PathBuf, Diagnostic> {
    let metadata = fs::metadata(source).map_err(|error| {
        let reason = if error.kind() == ErrorKind::NotFound {
            "is missing"
        } else {
            "could not be read"
        };
        broken(
            subject,
            State::Error,
            format!("source {} {reason}", source.display()),
        )
    })?;
    if !metadata.is_dir() {
        return Err(broken(
            subject,
            State::Error,
            format!("source {} is not a directory", source.display()),
        ));
    }
    canonical_within(source, root).ok_or_else(|| {
        broken(
            subject,
            State::Error,
            format!(
                "source {} resolves outside the repository",
                source.display()
            ),
        )
    })
}

fn same_target(path: &Path, expected: &Path, subject: &str) -> Result<(), Diagnostic> {
    match fs::canonicalize(path) {
        Ok(actual) if actual == expected => Ok(()),
        _ => Err(broken(
            subject,
            State::Drift,
            format!(
                "destination {} has the wrong symlink target",
                path.display()
            ),
        )),
    }
}

fn broken(subject: &str, state: State, reason: String) -> Diagnostic {
    Diagnostic::new("skills", state, format!("broken {subject}: {reason}"))
}
