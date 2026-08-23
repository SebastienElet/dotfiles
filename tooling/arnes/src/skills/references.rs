use crate::diagnostic::{Diagnostic, State};
use std::fs;
use std::path::{Component, Path, PathBuf};

mod parser;

use parser::local_references;

pub fn validate(skill: &Path, subject: &str) -> Result<(), Diagnostic> {
    let file = skill.join("SKILL.md");
    let contents = read_skill(skill, &file, subject)?;
    for reference in local_references(&contents) {
        validate_reference(skill, &reference, subject)?;
    }
    Ok(())
}

fn read_skill(skill: &Path, file: &Path, subject: &str) -> Result<String, Diagnostic> {
    let metadata = fs::metadata(file).map_err(|error| {
        let (state, reason) = if error.kind() == std::io::ErrorKind::NotFound {
            (State::Drift, "is missing")
        } else {
            (State::Error, "could not be read")
        };
        broken(subject, state, format!("{} {reason}", file.display()))
    })?;
    if !metadata.is_file() {
        return Err(broken(
            subject,
            State::Error,
            format!("{} is not a file", file.display()),
        ));
    }
    ensure_within(file, skill, subject)?;
    fs::read_to_string(file).map_err(|_| {
        broken(
            subject,
            State::Error,
            format!("{} could not be read", file.display()),
        )
    })
}

fn validate_reference(skill: &Path, reference: &Path, subject: &str) -> Result<(), Diagnostic> {
    let path = resolve(skill, reference).ok_or_else(|| {
        broken(
            subject,
            State::Error,
            format!("local resource {} escapes its skill", reference.display()),
        )
    })?;
    fs::metadata(&path).map_err(|error| {
        let (state, reason) = if error.kind() == std::io::ErrorKind::NotFound {
            (State::Drift, "is missing")
        } else {
            (State::Error, "could not be read")
        };
        broken(
            subject,
            state,
            format!("local resource {} {reason}", reference.display()),
        )
    })?;
    ensure_within(&path, skill, subject)
}

fn ensure_within(path: &Path, skill: &Path, subject: &str) -> Result<(), Diagnostic> {
    let root = fs::canonicalize(skill).map_err(|_| {
        broken(
            subject,
            State::Error,
            format!("skill {} could not be resolved", skill.display()),
        )
    })?;
    let resolved = fs::canonicalize(path).map_err(|_| {
        broken(
            subject,
            State::Error,
            format!("{} could not be resolved", path.display()),
        )
    })?;
    if resolved.starts_with(root) {
        Ok(())
    } else {
        Err(broken(
            subject,
            State::Error,
            format!("{} resolves outside its skill", path.display()),
        ))
    }
}

fn resolve(root: &Path, reference: &Path) -> Option<PathBuf> {
    let mut path = root.to_owned();
    for component in reference.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => path.push(part),
            Component::ParentDir if path != root => {
                path.pop();
            }
            _ => return None,
        }
    }
    Some(path)
}

fn broken(subject: &str, state: State, reason: String) -> Diagnostic {
    Diagnostic::new("skills", state, format!("broken {subject}: {reason}"))
}
