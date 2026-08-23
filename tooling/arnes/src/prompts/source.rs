use super::Failure;
use crate::Roots;
use crate::diagnostic::State;
use crate::files::includes::{self, Graph, IncludeError, Resolver};
use crate::manifest::Prompt;
use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};

use super::variables;

pub struct Expected {
    pub direct: String,
    pub rendered: String,
}

pub fn validate(roots: &Roots, prompt: Prompt<'_>) -> Result<Expected, Failure> {
    let source = roots.repository().join(prompt.source());
    let resolver = Resolver::new(roots.repository());
    let direct = resolver
        .read(&source)
        .map_err(|error| source_failure(roots, error))?;
    let graph = resolver
        .walk(&source)
        .map_err(|error| include_failure(roots, error))?;
    let includes = declared_includes(roots, prompt, &resolver)?;
    validate_include_identities(roots, &includes)?;
    validate_graph(roots, &source, &graph, &includes)?;
    validate_variables(roots, prompt, &resolver, &source, &includes)?;
    let rendered = render(roots, &resolver, &source)?;
    Ok(Expected { direct, rendered })
}

fn validate_include_identities(roots: &Roots, includes: &[PathBuf]) -> Result<(), Failure> {
    let mut identities = HashSet::new();
    for path in includes {
        let identity = crate::files::paths::canonical_within(path, roots.repository())
            .ok_or_else(|| include_failure(roots, IncludeError::OutsideRoot(path.clone())))?;
        if !identities.insert(identity) {
            return Err(Failure::new(
                State::Error,
                format!(
                    "declared include {} aliases another include",
                    relative(roots, path)
                ),
                "ambiguous include",
            ));
        }
    }
    Ok(())
}

fn declared_includes(
    roots: &Roots,
    prompt: Prompt<'_>,
    resolver: &Resolver,
) -> Result<Vec<PathBuf>, Failure> {
    let source = roots.repository().join(prompt.source());
    let parent = source.parent().unwrap_or(roots.repository());
    prompt
        .includes()
        .map(|include| {
            let include = include.to_str().ok_or_else(|| {
                Failure::new(
                    State::Error,
                    "declared include path is not valid UTF-8",
                    "invalid include path",
                )
            })?;
            resolver
                .resolve(parent, include)
                .map_err(|error| include_failure(roots, error))
        })
        .collect()
}

fn validate_graph(
    roots: &Roots,
    source: &Path,
    graph: &Graph,
    declared: &[PathBuf],
) -> Result<(), Failure> {
    for path in declared {
        if !graph.contains(path) {
            return Err(Failure::new(
                State::Error,
                format!(
                    "declared include {} is not referenced",
                    relative(roots, path)
                ),
                "declared include not referenced",
            ));
        }
    }
    let declared = declared.iter().cloned().collect::<HashSet<_>>();
    let mut actual = graph
        .paths()
        .filter(|path| *path != source && !declared.contains(*path))
        .collect::<Vec<_>>();
    actual.sort_unstable();
    if let Some(path) = actual.first() {
        return Err(Failure::new(
            State::Error,
            format!(
                "referenced include {} is not declared",
                relative(roots, path)
            ),
            "referenced include not declared",
        ));
    }
    Ok(())
}

fn validate_variables(
    roots: &Roots,
    prompt: Prompt<'_>,
    resolver: &Resolver,
    source: &Path,
    includes: &[PathBuf],
) -> Result<(), Failure> {
    let declared = prompt.variables().collect::<HashSet<_>>();
    let mut undeclared = BTreeSet::new();
    for path in std::iter::once(source).chain(includes.iter().map(PathBuf::as_path)) {
        let contents = resolver
            .read(path)
            .map_err(|error| include_failure(roots, error))?;
        let references = variables::references(&contents).map_err(|()| {
            Failure::new(
                State::Error,
                format!("invalid variable reference in {}", relative(roots, path)),
                "invalid variable reference",
            )
        })?;
        undeclared.extend(
            references
                .into_iter()
                .filter(|variable| !declared.contains(variable.as_str())),
        );
    }
    if undeclared.is_empty() {
        Ok(())
    } else {
        let names = undeclared.into_iter().collect::<Vec<_>>().join(", ");
        Err(Failure::new(
            State::Error,
            format!("variables {names} are referenced but not declared"),
            "undeclared variables",
        ))
    }
}

fn render(roots: &Roots, resolver: &Resolver, path: &Path) -> Result<String, Failure> {
    let contents = resolver
        .read(path)
        .map_err(|error| include_failure(roots, error))?;
    let mut rendered = includes::without_leading_imports(&contents);
    for include in includes::leading_imports(&contents) {
        let path = resolver
            .resolve(path.parent().unwrap_or(roots.repository()), &include)
            .map_err(|error| include_failure(roots, error))?;
        rendered.push_str(&render(roots, resolver, &path)?);
    }
    Ok(rendered)
}

fn source_failure(roots: &Roots, error: IncludeError) -> Failure {
    let message = match error {
        IncludeError::Missing(path) => format!("source {} is missing", relative(roots, &path)),
        IncludeError::MissingLink(path) | IncludeError::Dangling(path) => {
            format!("source {} is a dangling symlink", relative(roots, &path))
        }
        IncludeError::NotFile(path) => {
            format!("source {} is not a regular file", relative(roots, &path))
        }
        IncludeError::Unreadable(path) => {
            format!("source {} could not be read", relative(roots, &path))
        }
        IncludeError::OutsideRoot(path) => format!(
            "source {} resolves outside the repository",
            relative(roots, &path)
        ),
        IncludeError::Escapes(path) => format!("source include {path} escapes the repository"),
        IncludeError::Cycle(path) => {
            format!("source include cycle reaches {}", relative(roots, &path))
        }
        IncludeError::WrongLink(path) => {
            format!("source {} has an invalid symlink", relative(roots, &path))
        }
    };
    Failure::new(State::Error, message, "invalid source")
}

fn include_failure(roots: &Roots, error: IncludeError) -> Failure {
    let message = match error {
        IncludeError::Missing(path) => format!("include {} is missing", relative(roots, &path)),
        IncludeError::MissingLink(path) | IncludeError::Dangling(path) => {
            format!("include {} is a dangling symlink", relative(roots, &path))
        }
        IncludeError::NotFile(path) => {
            format!("include {} is not a regular file", relative(roots, &path))
        }
        IncludeError::Unreadable(path) => {
            format!("include {} could not be read", relative(roots, &path))
        }
        IncludeError::Escapes(path) => format!("include {path} escapes the repository"),
        IncludeError::OutsideRoot(path) => format!(
            "include {} resolves outside the repository",
            relative(roots, &path)
        ),
        IncludeError::Cycle(path) => {
            format!("include cycle reaches {}", relative(roots, &path))
        }
        IncludeError::WrongLink(path) => {
            format!("include {} has an invalid symlink", relative(roots, &path))
        }
    };
    Failure::new(State::Error, message, "invalid include graph")
}

fn relative(roots: &Roots, path: &Path) -> String {
    path.strip_prefix(roots.repository())
        .unwrap_or(path)
        .display()
        .to_string()
}
