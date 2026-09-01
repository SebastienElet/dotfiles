use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::files::paths::canonical_within;
use crate::manifest::{McpRegistration, Scope};
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub(super) fn diagnose(roots: &Roots, registration: McpRegistration<'_>) -> Option<Diagnostic> {
    let identity = format!(
        "{} {} {}",
        registration.agent, registration.scope, registration.name
    );
    let candidate = match candidate(roots, registration.scope, registration.command) {
        Ok(Some(path)) => path,
        Ok(None) => {
            return Some(Diagnostic::new(
                "mcp",
                State::Drift,
                format!("{identity}: command is missing"),
            ));
        }
        Err(reason) => {
            return Some(Diagnostic::new(
                "mcp",
                State::Error,
                format!("{identity}: {reason}"),
            ));
        }
    };
    if escapes_scope(roots, registration.scope, registration.command, &candidate) {
        return Some(Diagnostic::new(
            "mcp",
            State::Error,
            format!("{identity}: command escapes its scope root"),
        ));
    }
    match fs::metadata(&candidate) {
        Ok(metadata) if !metadata.is_file() => Some(Diagnostic::new(
            "mcp",
            State::Error,
            format!("{identity}: command is not a file"),
        )),
        Ok(metadata) if metadata.permissions().mode() & 0o111 == 0 => Some(Diagnostic::new(
            "mcp",
            State::Error,
            format!("{identity}: command is not executable"),
        )),
        Ok(_) => None,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Some(Diagnostic::new(
            "mcp",
            State::Drift,
            format!("{identity}: command is missing"),
        )),
        Err(_) => Some(Diagnostic::new(
            "mcp",
            State::Error,
            format!("{identity}: command metadata is unreadable"),
        )),
    }
}

fn escapes_scope(roots: &Roots, scope: Scope, command: &str, candidate: &Path) -> bool {
    if Path::new(command).is_absolute()
        || !command.contains(std::path::MAIN_SEPARATOR)
        || fs::symlink_metadata(candidate).is_err()
    {
        return false;
    }
    let root = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    canonical_within(candidate, root).is_none()
}

fn candidate(roots: &Roots, scope: Scope, command: &str) -> Result<Option<PathBuf>, &'static str> {
    if command.contains(std::path::MAIN_SEPARATOR) {
        let path = PathBuf::from(command);
        let root = match scope {
            Scope::User => roots.home(),
            Scope::Project => roots.repository(),
        };
        return Ok(Some(if path.is_absolute() {
            path
        } else {
            root.join(path)
        }));
    }
    let path = env::var_os("PATH").ok_or("PATH is unavailable")?;
    let mut invalid = None;
    for directory in env::split_paths(&path) {
        let candidate = directory.join(command);
        match fs::metadata(&candidate) {
            Ok(metadata) if metadata.is_file() && metadata.permissions().mode() & 0o111 != 0 => {
                return Ok(Some(candidate));
            }
            Ok(metadata) if !metadata.is_file() => {
                invalid.get_or_insert("command in PATH is not a file");
            }
            Ok(_) => {
                invalid.get_or_insert("command in PATH is not executable");
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("PATH entry is unreadable"),
        }
    }
    match invalid {
        Some(error) => Err(error),
        None => Ok(None),
    }
}
