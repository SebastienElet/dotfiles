use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::{ancestor_within, canonical_within, same_file};
use crate::manifest::{Agent, Manifest, RuleResource, Scope};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

pub fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    let combinations = manifest
        .combinations()
        .filter(|(candidate, _)| agent.is_none_or(|agent| agent == *candidate))
        .filter(|(_, candidate)| scope.is_none_or(|scope| scope == *candidate))
        .collect::<Vec<_>>();

    if combinations.is_empty() {
        return vec![unsupported(agent, scope)];
    }

    combinations
        .into_iter()
        .flat_map(|(agent, scope)| {
            let resources = manifest
                .rule_resources()
                .filter(|resource| resource.agent == agent && resource.scope == scope)
                .collect::<Vec<_>>();
            if !supported(agent, scope) || resources.is_empty() {
                return vec![unsupported(Some(agent), Some(scope))];
            }
            resources
                .into_iter()
                .map(|resource| diagnose_resource(roots, resource))
                .collect()
        })
        .collect()
}

fn supported(agent: Agent, scope: Scope) -> bool {
    agent == Agent::Claude && scope == Scope::User
}

fn diagnose_resource(roots: &Roots, resource: RuleResource<'_>) -> Diagnostic {
    let source = roots.repository().join(resource.source);
    let destination = roots.home().join(resource.destination);
    let subject = format!("{} {} rule {}", resource.agent, resource.scope, resource.id);

    if let Err(diagnostic) = valid_source(&source, roots.repository(), &subject) {
        return diagnostic;
    }
    if let Err(diagnostic) = valid_destination(&destination, &source, roots.home(), &subject) {
        return diagnostic;
    }
    diagnostic(
        State::Healthy,
        &subject,
        format!(
            "destination ~/{} is current",
            resource.destination.display()
        ),
    )
}

fn valid_destination(
    destination: &Path,
    source: &Path,
    root: &Path,
    subject: &str,
) -> Result<(), Diagnostic> {
    if !destination
        .parent()
        .is_some_and(|parent| ancestor_within(parent, root))
    {
        return Err(diagnostic(
            State::Error,
            subject,
            format!(
                "destination {} resolves outside its declared root",
                destination.display()
            ),
        ));
    }
    let metadata = destination_metadata(destination, subject)?;
    if !metadata.file_type().is_symlink() {
        return Err(diagnostic(
            State::Drift,
            subject,
            format!("destination {} is not a symlink", destination.display()),
        ));
    }
    followed_destination(destination, subject)?;
    if !same_file(destination, source) {
        return Err(diagnostic(
            State::Drift,
            subject,
            format!(
                "destination {} has the wrong symlink target",
                destination.display()
            ),
        ));
    }
    Ok(())
}

fn destination_metadata(destination: &Path, subject: &str) -> Result<fs::Metadata, Diagnostic> {
    match fs::symlink_metadata(destination) {
        Ok(metadata) => Ok(metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => Err(diagnostic(
            State::Drift,
            subject,
            format!("destination {} is missing", destination.display()),
        )),
        Err(_) => Err(diagnostic(
            State::Error,
            subject,
            format!("destination {} could not be read", destination.display()),
        )),
    }
}

fn followed_destination(destination: &Path, subject: &str) -> Result<(), Diagnostic> {
    match fs::metadata(destination) {
        Ok(_) => Ok(()),
        Err(error) if error.kind() == ErrorKind::NotFound => Err(diagnostic(
            State::Drift,
            subject,
            format!(
                "destination {} is a dangling symlink",
                destination.display()
            ),
        )),
        Err(_) => Err(diagnostic(
            State::Error,
            subject,
            format!(
                "destination {} could not be followed",
                destination.display()
            ),
        )),
    }
}

fn valid_source(source: &Path, root: &Path, subject: &str) -> Result<(), Diagnostic> {
    source_exists(source, subject)?;
    source_within(source, root, subject)?;
    let metadata = fs::metadata(source).map_err(|_| {
        diagnostic(
            State::Error,
            subject,
            format!("source {} could not be read", source.display()),
        )
    })?;
    if !metadata.file_type().is_file() {
        return Err(diagnostic(
            State::Error,
            subject,
            format!("source {} is not a regular file", source.display()),
        ));
    }
    fs::read_to_string(source).map_err(|_| {
        diagnostic(
            State::Error,
            subject,
            format!("source {} could not be read as text", source.display()),
        )
    })?;
    Ok(())
}

fn source_exists(source: &Path, subject: &str) -> Result<(), Diagnostic> {
    fs::symlink_metadata(source).map(|_| ()).map_err(|error| {
        let reason = if error.kind() == ErrorKind::NotFound {
            "is missing"
        } else {
            "could not be read"
        };
        diagnostic(
            State::Error,
            subject,
            format!("source {} {reason}", source.display()),
        )
    })
}

fn source_within(source: &Path, root: &Path, subject: &str) -> Result<(), Diagnostic> {
    if canonical_within(source, root).is_some() {
        return Ok(());
    }
    if fs::metadata(source).is_err_and(|error| error.kind() == ErrorKind::NotFound) {
        return Err(diagnostic(
            State::Error,
            subject,
            format!("source {} is missing (dangling symlink)", source.display()),
        ));
    }
    Err(diagnostic(
        State::Error,
        subject,
        format!(
            "source {} resolves outside the repository",
            source.display()
        ),
    ))
}

fn unsupported(agent: Option<Agent>, scope: Option<Scope>) -> Diagnostic {
    let subject = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} rule projection"),
        (Some(agent), None) => format!("{agent} rule projection"),
        (None, Some(scope)) => format!("{scope} rule scope"),
        (None, None) => "rule projections".to_owned(),
    };
    Diagnostic::new(
        "rules",
        State::Unsupported,
        format!("{subject} is not declared or supported"),
    )
}

fn diagnostic(state: State, subject: &str, message: String) -> Diagnostic {
    Diagnostic::new("rules", state, format!("{subject}: {message}"))
}
