use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Scope};
use std::fmt::{self, Display};
use std::fs;
use std::io::ErrorKind;
use std::path::Path;

mod defaults;
mod format;

use format::{ConfigFormat, ParseError};

struct Specification {
    root: &'static str,
    file: &'static str,
    format: ConfigFormat,
}

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
        return vec![Diagnostic::new(
            "config",
            State::Unsupported,
            unsupported_message(agent, scope),
        )];
    }

    combinations
        .into_iter()
        .map(|(agent, scope)| diagnose_one(roots, manifest, agent, scope))
        .collect()
}

fn diagnose_one(roots: &Roots, manifest: &Manifest, agent: Agent, scope: Scope) -> Diagnostic {
    let specification = specification(agent, scope);
    let base = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let root = base.join(specification.root);
    let root_label = match scope {
        Scope::User => format!("~/{}", specification.root),
        Scope::Project => specification.root.to_owned(),
    };
    let file = root.join(specification.file);
    let file_label = format!("{root_label}/{}", specification.file);
    let subject = format!("{agent} {scope} configuration");

    if let Err(diagnostic) = check_root(&root, &subject, &root_label) {
        return diagnostic;
    }
    let input = match read_file(&file, &subject, &file_label) {
        Ok(input) => input,
        Err(diagnostic) => return diagnostic,
    };

    match format::parse(&input, specification.format) {
        Ok(value) => diagnose_defaults(manifest, agent, scope, &value, &subject, &file_label),
        Err(ParseError::Malformed) => Diagnostic::new(
            "config",
            State::Error,
            format!(
                "{subject} file {file_label} contains malformed {}",
                specification.format
            ),
        ),
        Err(ParseError::WrongRoot) => Diagnostic::new(
            "config",
            State::Error,
            format!(
                "{subject} file {file_label} must contain a top-level {} object",
                specification.format
            ),
        ),
    }
}

fn diagnose_defaults(
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
    value: &serde_json::Value,
    subject: &str,
    file_label: &str,
) -> Diagnostic {
    let mismatches = match scope {
        Scope::User => manifest
            .user_config(agent)
            .map(|config| defaults::mismatches(agent, config, value))
            .unwrap_or_default(),
        Scope::Project => Vec::new(),
    };
    if mismatches.is_empty() {
        Diagnostic::new(
            "config",
            State::Healthy,
            format!("{subject} file {file_label} is valid"),
        )
    } else {
        Diagnostic::new(
            "config",
            State::Drift,
            format!(
                "{subject} file {file_label} differs from manifest defaults: {}",
                mismatches.join(", ")
            ),
        )
    }
}

fn check_root(root: &Path, subject: &str, root_label: &str) -> Result<(), Diagnostic> {
    match fs::metadata(root) {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(Diagnostic::new(
                "config",
                State::Drift,
                format!("{subject} root {root_label} is missing"),
            ));
        }
        Err(_) => return Err(unreadable(subject, "root", root_label)),
        Ok(metadata) if !metadata.is_dir() => {
            return Err(Diagnostic::new(
                "config",
                State::Error,
                format!("{subject} root {root_label} is not a directory"),
            ));
        }
        Ok(_) => {}
    }
    if fs::read_dir(root).is_err() {
        return Err(unreadable(subject, "root", root_label));
    }
    Ok(())
}

fn read_file(file: &Path, subject: &str, file_label: &str) -> Result<String, Diagnostic> {
    match fs::metadata(file) {
        Err(error) if error.kind() == ErrorKind::NotFound => {
            return Err(Diagnostic::new(
                "config",
                State::Drift,
                format!("{subject} file {file_label} is missing"),
            ));
        }
        Err(_) => return Err(unreadable(subject, "file", file_label)),
        Ok(metadata) if !metadata.is_file() => {
            return Err(Diagnostic::new(
                "config",
                State::Error,
                format!("{subject} file {file_label} is not a file"),
            ));
        }
        Ok(_) => {}
    }
    fs::read_to_string(file).map_err(|_| unreadable(subject, "file", file_label))
}

fn unreadable(subject: &str, kind: &str, path: &str) -> Diagnostic {
    Diagnostic::new(
        "config",
        State::Error,
        format!("{subject} {kind} {path} could not be read"),
    )
}

fn specification(agent: Agent, scope: Scope) -> Specification {
    match (agent, scope) {
        (Agent::Claude, _) => Specification {
            root: ".claude",
            file: "settings.json",
            format: ConfigFormat::Json,
        },
        (Agent::Cursor, Scope::User) => Specification {
            root: ".cursor",
            file: "cli-config.json",
            format: ConfigFormat::Json,
        },
        (Agent::Cursor, Scope::Project) => Specification {
            root: ".cursor",
            file: "cli.json",
            format: ConfigFormat::Json,
        },
        (Agent::Codex, _) => Specification {
            root: ".codex",
            file: "config.toml",
            format: ConfigFormat::Toml,
        },
    }
}

fn unsupported_message(agent: Option<Agent>, scope: Option<Scope>) -> String {
    match (agent, scope) {
        (Some(agent), Some(scope)) => {
            format!("{agent} {scope} configuration is not declared in the manifest")
        }
        (Some(agent), None) => {
            format!("{agent} configuration is not declared in the manifest")
        }
        (None, Some(scope)) => {
            format!("{scope} configuration scope is not declared in the manifest")
        }
        (None, None) => "no configuration combinations are declared in the manifest".to_owned(),
    }
}

impl Display for Agent {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Claude => "claude",
            Self::Cursor => "cursor",
            Self::Codex => "codex",
        })
    }
}

impl Display for Scope {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::User => "user",
            Self::Project => "project",
        })
    }
}
