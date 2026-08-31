use super::adapters::{self, Policy};
use super::{json_value, validate};
use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, HookKind, Manifest, Scope};
use serde_json::Value;
use std::fs;
use std::io::ErrorKind;

mod comparison;
mod expectation;
mod presence;

const KINDS: [HookKind; 2] = [HookKind::Measurement, HookKind::Handoff];

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
        return vec![unsupported(undeclared_subject(agent, scope))];
    }

    combinations
        .into_iter()
        .flat_map(|(agent, scope)| diagnose_one(roots, manifest, agent, scope))
        .collect()
}

fn diagnose_one(roots: &Roots, manifest: &Manifest, agent: Agent, scope: Scope) -> Vec<Diagnostic> {
    let subject = format!("{agent} {scope}");
    if scope != Scope::User {
        return vec![unsupported(format!("{subject} hooks are not supported"))];
    }
    let declared = manifest.hooks(agent, scope).collect::<Vec<_>>();
    let policy = adapters::policy(agent);
    let label = format!("~/{}/{}", policy.directory, policy.filename);
    let config = match load(roots, &policy, agent) {
        Ok(Some(config)) => config,
        Ok(None) if declared.is_empty() => {
            return vec![unsupported(format!("{subject} hooks are not declared"))];
        }
        Ok(None) => {
            return vec![drift(format!(
                "{subject} hook configuration {label} is missing"
            ))];
        }
        Err(reason) => {
            return vec![error(format!(
                "{subject} hook configuration {label} {reason}"
            ))];
        }
    };

    let mut diagnostics = Vec::new();
    if declared.is_empty() {
        diagnostics.push(unsupported(format!("{subject} hooks are not declared")));
    }
    diagnostics.extend(KINDS.into_iter().filter_map(|kind| {
        diagnose_kind(
            roots,
            &config,
            &policy,
            agent,
            kind,
            declared.contains(&kind),
            &subject,
        )
    }));
    diagnostics
}

fn load(roots: &Roots, policy: &Policy, agent: Agent) -> Result<Option<Value>, String> {
    let file = roots.home().join(policy.directory).join(policy.filename);
    let bytes = match fs::read(&file) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err("could not be read".to_owned()),
    };
    let config = json_value::parse(&bytes).map_err(|_| "is malformed".to_owned())?;
    validate::configuration(&config, agent).map_err(|error| format!("is invalid: {error}"))?;
    Ok(Some(config))
}

fn diagnose_kind(
    roots: &Roots,
    config: &Value,
    policy: &Policy,
    agent: Agent,
    kind: HookKind,
    declared: bool,
    subject: &str,
) -> Option<Diagnostic> {
    let expectation = match expectation::expectation(roots, policy, agent, kind) {
        Ok(expectation) => expectation,
        Err(reason) => return Some(error(format!("{subject} {kind} hook: {reason}"))),
    };
    let installed = presence::events(config, expectation.nested, &expectation.command);
    let superseded = comparison::superseded(config, &expectation);
    if !declared {
        return (!installed.is_empty() || !superseded.is_empty()).then(|| {
            drift(format!(
                "{subject} {kind} hook is installed but not declared"
            ))
        });
    }
    if let Some(diagnostic) = expectation.command_state(subject, kind) {
        return Some(diagnostic);
    }
    Some(comparison::compare(
        &expectation,
        &installed,
        &superseded,
        subject,
        kind,
    ))
}

fn undeclared_subject(agent: Option<Agent>, scope: Option<Scope>) -> String {
    let subject = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} hook installations"),
        (Some(agent), None) => format!("{agent} hook installations"),
        (None, Some(scope)) => format!("{scope} hook scope"),
        (None, None) => "hook installations".to_owned(),
    };
    format!("{subject} are not declared or supported")
}

fn healthy(message: String) -> Diagnostic {
    Diagnostic::new("hooks", State::Healthy, message)
}

fn drift(message: String) -> Diagnostic {
    Diagnostic::new("hooks", State::Drift, message)
}

fn error(message: String) -> Diagnostic {
    Diagnostic::new("hooks", State::Error, message)
}

fn unsupported(message: String) -> Diagnostic {
    Diagnostic::new("hooks", State::Unsupported, message)
}
