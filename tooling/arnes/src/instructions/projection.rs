use super::checks::{
    destination, destination_label, expected_file, expected_link, healthy, include_diagnostic,
    relative, source_diagnostic,
};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::includes::{self, Resolver};
use crate::manifest::{Agent, InstructionResource, Scope};
use std::fs;
use std::path::Path;

#[derive(Clone, Copy)]
pub enum Projection {
    Link,
    Include,
    Generated,
}

pub fn kind(agent: Agent, scope: Scope) -> Option<Projection> {
    match (agent, scope) {
        (Agent::Claude, Scope::User) => Some(Projection::Link),
        (Agent::Claude, Scope::Project) => Some(Projection::Include),
        (Agent::Codex, Scope::User) => Some(Projection::Generated),
        _ => None,
    }
}

pub fn diagnose(
    roots: &Roots,
    resource: &InstructionResource<'_>,
    resources: &[InstructionResource<'_>],
    projection: Projection,
) -> Diagnostic {
    let source_root = match resource.scope {
        Scope::User => roots.deployment_repository(),
        Scope::Project => roots.repository(),
    };
    let source = source_root.join(resource.source);
    let destination = destination(roots, resource);
    let subject = format!(
        "{} {} instructions {}",
        resource.agent, resource.scope, resource.id
    );
    let source_contents = match includes::read_regular(&source, source_root) {
        Ok(contents) => contents,
        Err(error) => return source_diagnostic(&subject, error),
    };

    match projection {
        Projection::Link => diagnose_link(
            roots,
            resource,
            resources,
            source_root,
            &source,
            &destination,
            &subject,
        ),
        Projection::Include => diagnose_include(roots, &source, &destination, &subject),
        Projection::Generated => diagnose_generated(
            roots,
            source_root,
            &source,
            &destination,
            &source_contents,
            &subject,
        ),
    }
}

fn diagnose_link(
    roots: &Roots,
    resource: &InstructionResource<'_>,
    resources: &[InstructionResource<'_>],
    source_root: &Path,
    source: &Path,
    destination: &Path,
    subject: &str,
) -> Diagnostic {
    if let Err(diagnostic) = expected_link(destination, source, roots.home(), subject) {
        return diagnostic;
    }
    let aliases = resources.iter().map(|candidate| {
        (
            roots.home().join(candidate.destination),
            source_root.join(candidate.source),
        )
    });
    let resolver = Resolver::with_aliases(roots.home(), source_root, aliases);
    match resolver.walk(destination) {
        Ok(_) => healthy(subject, destination_label(resource)),
        Err(error) => include_diagnostic(subject, error, State::Error),
    }
}

fn diagnose_include(roots: &Roots, source: &Path, destination: &Path, subject: &str) -> Diagnostic {
    if let Err(diagnostic) = expected_file(destination, roots.repository(), subject) {
        return diagnostic;
    }
    let resolver = Resolver::new(roots.repository());
    match resolver.walk(destination) {
        Ok(graph) if graph.contains(source) => healthy(subject, relative(destination, roots)),
        Ok(_) => Diagnostic::new(
            "instructions",
            State::Drift,
            format!(
                "{subject} does not include source {}",
                relative(source, roots)
            ),
        ),
        Err(error) => include_diagnostic(subject, error, State::Error),
    }
}

fn diagnose_generated(
    roots: &Roots,
    source_root: &Path,
    source: &Path,
    destination: &Path,
    source_contents: &str,
    subject: &str,
) -> Diagnostic {
    if let Err(diagnostic) = expected_file(destination, roots.home(), subject) {
        return diagnostic;
    }
    let resolver = Resolver::new(source_root);
    if let Err(error) = resolver.walk(source) {
        return include_diagnostic(subject, error, State::Error);
    }
    let mut expected = includes::without_leading_imports(source_contents);
    for include in includes::leading_imports(source_contents) {
        let path = match resolver.resolve(source.parent().unwrap_or(source_root), &include) {
            Ok(path) => path,
            Err(error) => return include_diagnostic(subject, error, State::Error),
        };
        match resolver.read(&path) {
            Ok(contents) => expected.push_str(&contents),
            Err(error) => return include_diagnostic(subject, error, State::Error),
        }
    }
    match fs::read_to_string(destination) {
        Ok(contents) if contents == expected => healthy(subject, relative(destination, roots)),
        Ok(_) => Diagnostic::new(
            "instructions",
            State::Drift,
            format!(
                "{subject} generated file {} is stale",
                relative(destination, roots)
            ),
        ),
        Err(_) => Diagnostic::new(
            "instructions",
            State::Error,
            format!(
                "{subject} file {} could not be read",
                relative(destination, roots)
            ),
        ),
    }
}
