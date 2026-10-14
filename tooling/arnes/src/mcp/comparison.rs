use super::observed::{EnvironmentValue, ObservedRegistration};
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{McpRegistration, McpTransport};

pub fn diagnose(expected: McpRegistration<'_>, observed: &ObservedRegistration) -> Vec<Diagnostic> {
    let identity = format!("{} {} {}", expected.agent, expected.scope, expected.name);
    let mut diagnostics = Vec::new();
    match (expected.transport, observed.url.as_deref()) {
        (
            McpTransport::Stdio {
                command,
                args,
                environment,
            },
            None,
        ) => {
            if command != observed.command {
                diagnostics.push(drift(&identity, "command differs"));
            }
            if args != observed.args {
                diagnostics.push(drift(&identity, "ordered arguments differ"));
            }
            if !environment_matches(environment, observed) {
                diagnostics.push(drift(&identity, "environment references differ"));
            }
        }
        (McpTransport::Http { url }, Some(actual)) => {
            if url != actual {
                diagnostics.push(drift(&identity, "URL differs"));
            }
        }
        _ => diagnostics.push(drift(&identity, "transport differs")),
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
