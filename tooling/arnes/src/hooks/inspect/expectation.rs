use super::super::adapters::Policy;
use super::super::{
    HooksError, MatchedHandler, OUTPUT_DISCIPLINE, format_edited_file_handler,
    format_edited_file_path, handoff_aliases, handoff_path, measurement_command, measurement_path,
    memory_command, memory_path, output_discipline_command, quoted_command,
};
use super::{drift, error};
use crate::Roots;
use crate::diagnostic::Diagnostic;
use crate::manifest::{Agent, HookKind};
use std::fs;
use std::io::ErrorKind;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

const HANDOFF_EVENTS: &[&str] = &["Stop"];

pub struct Expectation {
    pub events: Vec<&'static str>,
    pub nested: bool,
    pub command: String,
    pub superseded: Vec<String>,
    pub settings: Option<MatchedHandler>,
    path: PathBuf,
    label: &'static str,
}

pub fn expectation(
    roots: &Roots,
    policy: &Policy,
    agent: Agent,
    kind: HookKind,
) -> Result<Expectation, HooksError> {
    match kind {
        HookKind::OutputDiscipline => {
            let path = measurement_path(roots.home());
            Ok(Expectation {
                events: vec!["SessionStart"],
                nested: true,
                command: output_discipline_command(&path)?,
                superseded: Vec::new(),
                settings: Some(OUTPUT_DISCIPLINE),
                path,
                label: "~/.local/bin/arnes",
            })
        }
        HookKind::Measurement => {
            let path = measurement_path(roots.home());
            Ok(Expectation {
                events: policy.events.to_vec(),
                nested: policy.nested,
                command: measurement_command(&path, agent)?,
                superseded: Vec::new(),
                settings: None,
                path,
                label: "~/.local/bin/arnes",
            })
        }
        HookKind::Handoff => {
            let path = handoff_path(roots.home());
            let mut aliases = handoff_aliases(&path, roots.repository(), agent)?.into_iter();
            let command = aliases
                .next()
                .ok_or_else(|| HooksError::new("handoff hook command is required"))?;
            Ok(Expectation {
                events: HANDOFF_EVENTS.to_vec(),
                nested: true,
                command,
                superseded: aliases.collect(),
                settings: None,
                path,
                label: "~/.local/bin/agent-handoff",
            })
        }
        HookKind::Memory => {
            let path = memory_path(roots.home());
            let command = memory_command(&path, agent)?
                .ok_or_else(|| HooksError::new("Cursor does not support the memory hook"))?;
            let event = policy
                .memory_event
                .ok_or_else(|| HooksError::new("Cursor does not support the memory hook"))?;
            Ok(Expectation {
                events: vec![event],
                nested: policy.nested,
                command,
                superseded: Vec::new(),
                settings: None,
                path,
                label: "~/.local/bin/agent-memory",
            })
        }
        HookKind::FormatEditedFile => {
            let path = format_edited_file_path(roots.deployment_repository());
            Ok(Expectation {
                events: vec![super::super::FORMAT_EDITED_FILE_EVENT],
                nested: policy.nested,
                command: quoted_command(&path)?,
                superseded: Vec::new(),
                settings: Some(format_edited_file_handler(agent)?),
                path,
                label: "tooling/format-edited-file",
            })
        }
    }
}

impl Expectation {
    pub fn command_state(&self, subject: &str, kind: HookKind) -> Option<Diagnostic> {
        let prefix = format!("{subject} {kind} hook command {}", self.label);
        let metadata = match fs::metadata(&self.path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => {
                return Some(drift(format!("{prefix} is missing")));
            }
            Err(_) => return Some(error(format!("{prefix} could not be read"))),
        };
        (!metadata.is_file() || metadata.permissions().mode() & 0o111 == 0)
            .then(|| error(format!("{prefix} is not an executable file")))
    }
}
