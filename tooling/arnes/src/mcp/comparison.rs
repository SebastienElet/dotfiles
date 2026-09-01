use super::observed::{EnvironmentValue, ObservedRegistration};
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::McpRegistration;

pub(super) fn diagnose(
    expected: McpRegistration<'_>,
    observed: &ObservedRegistration,
) -> Vec<Diagnostic> {
    let identity = format!("{} {} {}", expected.agent, expected.scope, expected.name);
    let mut diagnostics = Vec::new();
    if expected.command != observed.command {
        diagnostics.push(drift(&identity, "command differs"));
    }
    if expected.args != observed.args {
        diagnostics.push(drift(&identity, "ordered arguments differ"));
    }
    if !environment_matches(expected.environment, observed) {
        diagnostics.push(drift(&identity, "environment references differ"));
    }
    if expected.enabled.is_some() && expected.enabled != observed.enabled {
        diagnostics.push(drift(&identity, "enabled state differs"));
    }
    diagnostics
}

fn environment_matches(expected: &[String], observed: &ObservedRegistration) -> bool {
    expected.len() == observed.environment.len()
        && expected.iter().all(|name| {
            observed.environment.get(name) == Some(&EnvironmentValue::Reference(name.clone()))
        })
}

fn drift(identity: &str, reason: &str) -> Diagnostic {
    Diagnostic::new("mcp", State::Drift, format!("{identity}: {reason}"))
}
