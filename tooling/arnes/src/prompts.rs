use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Prompt, PromptProjection, PromptRepresentation, Scope};

pub(crate) mod capability;
mod projection;
mod source;
mod topology;
mod variables;

pub(crate) use topology::Tracker as ProjectionTracker;

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
        return vec![unsupported_combination(agent, scope)];
    }

    let prompts = manifest.prompts().collect::<Vec<_>>();
    let selected_scopes = combinations
        .iter()
        .filter(|(agent, scope)| capability::registry(*agent, *scope).is_some())
        .map(|(_, scope)| *scope)
        .collect::<Vec<_>>();
    let scopes = ProjectionTracker::relevant_scopes(roots, &selected_scopes);
    let mut diagnostics = Vec::new();
    let mut topology =
        (!scopes.is_empty()).then(|| ProjectionTracker::new_for_scopes(roots, manifest, &scopes));
    for (agent, scope) in combinations {
        if capability::registry(agent, scope).is_none() {
            diagnostics.push(unsupported_combination(Some(agent), Some(scope)));
            continue;
        }
        let mut projected = false;
        for prompt in &prompts {
            for projection in prompt
                .projections()
                .filter(|projection| projection.agent == agent && projection.scope == scope)
            {
                projected = true;
                let topology = topology
                    .as_mut()
                    .expect("supported prompt projections initialize topology");
                if let Err(failure) = topology.validate(roots, *prompt, projection) {
                    diagnostics.push(broken(*prompt, projection, failure));
                } else {
                    diagnostics.push(diagnose_projection(roots, *prompt, projection));
                }
            }
        }
        if !projected {
            diagnostics.push(unsupported_combination(Some(agent), Some(scope)));
        }
    }
    diagnostics
}

fn diagnose_projection(
    roots: &Roots,
    prompt: Prompt<'_>,
    projection: PromptProjection<'_>,
) -> Diagnostic {
    match validate_projection(roots, prompt, projection) {
        Ok(_) => diagnostic(
            prompt,
            projection,
            State::Healthy,
            format!("{} is current", subject(prompt, projection)),
            "current",
        ),
        Err(failure) => broken(prompt, projection, failure),
    }
}

pub(crate) fn validate_projection(
    roots: &Roots,
    prompt: Prompt<'_>,
    projection: PromptProjection<'_>,
) -> Result<String, Failure> {
    if projection.representation == PromptRepresentation::Symlink {
        return Err(Failure::new(
            State::Unsupported,
            "symlink projections have no stable agent contract",
            "symlink projection unsupported",
        ));
    }
    let expected = source::validate(roots, prompt)?;
    projection::validate(roots, projection, &expected)
}

fn unsupported_combination(agent: Option<Agent>, scope: Option<Scope>) -> Diagnostic {
    let subject = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} prompt projection"),
        (Some(agent), None) => format!("{agent} prompt projections"),
        (None, Some(scope)) => format!("{scope} prompt scope"),
        (None, None) => "prompt projections".to_owned(),
    };
    let group = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} prompts"),
        _ => "prompts".to_owned(),
    };
    Diagnostic::new(
        "prompts",
        State::Unsupported,
        format!("{subject} is not declared or has no stable reusable-prompt contract"),
    )
    .with_human(group, "capability · unsupported")
}

fn broken(prompt: Prompt<'_>, projection: PromptProjection<'_>, failure: Failure) -> Diagnostic {
    diagnostic(
        prompt,
        projection,
        failure.state,
        format!("{}: {}", subject(prompt, projection), failure.message),
        failure.summary,
    )
}

fn diagnostic(
    prompt: Prompt<'_>,
    projection: PromptProjection<'_>,
    state: State,
    message: impl Into<String>,
    summary: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new("prompts", state, message).with_human(
        format!("{} {} prompts", projection.agent, projection.scope),
        format!("{} · {}", prompt.id(), summary.into()),
    )
}

fn subject(prompt: Prompt<'_>, projection: PromptProjection<'_>) -> String {
    format!(
        "managed {} {} prompt {} at {}",
        projection.agent,
        projection.scope,
        prompt.id(),
        crate::files::paths::label(projection.scope, projection.destination)
    )
}

pub(crate) struct Failure {
    pub(crate) state: State,
    pub(crate) message: String,
    pub(crate) summary: String,
}

impl Failure {
    pub(crate) fn new(
        state: State,
        message: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            state,
            message: message.into(),
            summary: summary.into(),
        }
    }
}
