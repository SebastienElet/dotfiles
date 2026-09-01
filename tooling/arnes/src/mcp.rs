mod command;
mod comparison;
mod configuration;
mod json;
mod observed;

use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, McpRegistration, Scope};

pub fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    let registrations = manifest
        .mcp_registrations()
        .filter(|registration| agent.is_none_or(|agent| registration.agent == agent))
        .filter(|registration| scope.is_none_or(|scope| registration.scope == scope))
        .collect::<Vec<_>>();
    if registrations.is_empty() && (agent.is_some() || scope.is_some()) {
        return vec![Diagnostic::new(
            "mcp",
            State::Unsupported,
            unsupported(agent, scope),
        )];
    }
    registrations
        .into_iter()
        .flat_map(|registration| diagnose_registration(roots, manifest, registration))
        .collect()
}

fn diagnose_registration(
    roots: &Roots,
    manifest: &Manifest,
    registration: McpRegistration<'_>,
) -> Vec<Diagnostic> {
    let identity = format!(
        "{} {} {}",
        registration.agent, registration.scope, registration.name
    );
    let configuration = match configuration::load(
        roots,
        registration.agent,
        registration.scope,
        &[registration.name],
    ) {
        Ok(Some(configuration)) => configuration,
        Ok(None) => {
            return vec![Diagnostic::new(
                "mcp",
                State::Drift,
                format!("{identity}: configuration is missing"),
            )];
        }
        Err(error) => {
            return vec![Diagnostic::new(
                "mcp",
                State::Error,
                format!("{identity}: {error}"),
            )];
        }
    };
    let Some(observed) = configuration.registrations.get(registration.name) else {
        return missing_registration(roots, manifest, registration);
    };
    let mut diagnostics = comparison::diagnose(registration, observed);
    diagnostics.extend(scope_collision(roots, manifest, registration));
    diagnostics.extend(command::diagnose(roots, registration));
    if diagnostics.is_empty() {
        diagnostics.push(Diagnostic::new(
            "mcp",
            State::Healthy,
            format!("{identity}: registration matches manifest"),
        ));
    }
    diagnostics
}

fn scope_collision(
    roots: &Roots,
    manifest: &Manifest,
    registration: McpRegistration<'_>,
) -> Option<Diagnostic> {
    let other_scope = match registration.scope {
        Scope::User => Scope::Project,
        Scope::Project => Scope::User,
    };
    if !manifest
        .combinations()
        .any(|pair| pair == (registration.agent, other_scope))
    {
        return None;
    }
    match configuration::load(roots, registration.agent, other_scope, &[registration.name]) {
        Ok(Some(configuration)) if configuration.registrations.contains_key(registration.name) => {
            Some(Diagnostic::new(
                "mcp",
                State::Drift,
                format!(
                    "{} {} {}: registration also exists in {other_scope} scope",
                    registration.agent, registration.scope, registration.name
                ),
            ))
        }
        Err(error) => Some(Diagnostic::new(
            "mcp",
            State::Error,
            format!(
                "{} {} {}: could not inspect {other_scope} scope: {error}",
                registration.agent, registration.scope, registration.name
            ),
        )),
        Ok(_) => None,
    }
}

fn missing_registration(
    roots: &Roots,
    manifest: &Manifest,
    registration: McpRegistration<'_>,
) -> Vec<Diagnostic> {
    let identity = format!(
        "{} {} {}",
        registration.agent, registration.scope, registration.name
    );
    let other_scope = match registration.scope {
        Scope::User => Scope::Project,
        Scope::Project => Scope::User,
    };
    let other = manifest
        .combinations()
        .any(|pair| pair == (registration.agent, other_scope))
        .then(|| configuration::load(roots, registration.agent, other_scope, &[registration.name]))
        .transpose();
    let collision = match other {
        Ok(Some(Some(configuration))) => {
            configuration.registrations.contains_key(registration.name)
        }
        Ok(_) => false,
        Err(error) => {
            return vec![Diagnostic::new(
                "mcp",
                State::Error,
                format!("{identity}: could not inspect {other_scope} scope: {error}"),
            )];
        }
    };
    let reason = if collision {
        "registration exists in the wrong scope"
    } else {
        "registration is missing"
    };
    vec![Diagnostic::new(
        "mcp",
        State::Drift,
        format!("{identity}: {reason}"),
    )]
}

fn unsupported(agent: Option<Agent>, scope: Option<Scope>) -> String {
    match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope}: no MCP registration is declared"),
        (Some(agent), None) => format!("{agent}: no MCP registration is declared"),
        (None, Some(scope)) => format!("{scope}: no MCP registration is declared"),
        (None, None) => "no MCP registration is declared".to_owned(),
    }
}
