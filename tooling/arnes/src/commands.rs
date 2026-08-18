use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, CommandBinding, Manifest, Prompt, PromptProjection, Scope};
use crate::prompts::{self, Failure, ProjectionTracker};
use std::collections::HashSet;
use std::path::Path;

mod binding;
mod capability;
#[cfg(test)]
mod tests;

pub fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    diagnose_with_tracker(roots, manifest, agent, scope, |scopes| {
        ProjectionTracker::new_for_scopes(roots, manifest, scopes)
    })
}

fn diagnose_with_tracker(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
    create_tracker: impl FnOnce(&[Scope]) -> ProjectionTracker,
) -> Vec<Diagnostic> {
    let selected = manifest
        .commands()
        .flat_map(|command| command.bindings())
        .filter(|binding| agent.is_none_or(|agent| agent == binding.agent))
        .filter(|binding| scope.is_none_or(|scope| scope == binding.scope))
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return vec![unsupported(agent, scope, None)];
    }
    let combinations = selected
        .iter()
        .filter(|binding| capability::supported(binding.agent, binding.scope))
        .map(|binding| (binding.agent, binding.scope))
        .collect::<HashSet<_>>();
    if combinations.is_empty() {
        return selected
            .into_iter()
            .map(|binding| unsupported_binding(binding))
            .collect();
    }
    let prompts = manifest.prompts().collect::<Vec<_>>();
    let selected_scopes = combinations
        .iter()
        .map(|(_, scope)| *scope)
        .collect::<Vec<_>>();
    let scopes = ProjectionTracker::relevant_scopes(roots, &selected_scopes);
    let topology_combinations = combinations
        .iter()
        .flat_map(|(agent, _)| scopes.iter().map(|scope| (*agent, *scope)))
        .collect::<HashSet<_>>();
    let mut topology = create_tracker(&scopes);
    seed_unbound_projections(
        roots,
        &selected,
        &prompts,
        &topology_combinations,
        &mut topology,
    );
    selected
        .into_iter()
        .map(|binding| diagnose_binding(roots, binding, &prompts, &mut topology))
        .collect()
}

fn seed_unbound_projections(
    roots: &Roots,
    selected: &[CommandBinding<'_>],
    prompts: &[Prompt<'_>],
    combinations: &HashSet<(Agent, Scope)>,
    topology: &mut ProjectionTracker,
) {
    let referenced = selected
        .iter()
        .filter(|binding| combinations.contains(&(binding.agent, binding.scope)))
        .map(|binding| (binding.prompt(), binding.agent, binding.scope))
        .collect::<HashSet<_>>();
    for prompt in prompts {
        for projection in prompt.projections().filter(|projection| {
            combinations.contains(&(projection.agent, projection.scope))
                && !referenced.contains(&(prompt.id(), projection.agent, projection.scope))
        }) {
            topology.seed_projection_destination(roots, *prompt, projection);
        }
    }
}

fn diagnose_binding(
    roots: &Roots,
    command: CommandBinding<'_>,
    prompts: &[Prompt<'_>],
    topology: &mut ProjectionTracker,
) -> Diagnostic {
    let Some(expected_destination) =
        capability::destination(command.agent, command.scope, command.name())
    else {
        return unsupported(
            Some(command.agent),
            Some(command.scope),
            Some(command.name()),
        );
    };
    let (prompt, projection) = match resolve_projection(command, prompts, &expected_destination) {
        Ok(binding) => binding,
        Err(diagnostic) => return diagnostic,
    };
    if let Err(failure) = topology.validate(roots, prompt, projection) {
        return broken(command, failure);
    }
    match prompts::validate_projection(roots, prompt, projection) {
        Err(failure) => broken(command, failure),
        Ok(contents) => match binding::validate(&contents, command.description()) {
            Ok(()) => diagnostic(command, State::Healthy, "binding is current", "current"),
            Err(message) => diagnostic(command, State::Drift, message, "binding stale"),
        },
    }
}

fn resolve_projection<'a>(
    command: CommandBinding<'a>,
    prompts: &[Prompt<'a>],
    expected_destination: &Path,
) -> Result<(Prompt<'a>, PromptProjection<'a>), Diagnostic> {
    let Some(prompt) = prompts
        .iter()
        .copied()
        .find(|prompt| prompt.id() == command.prompt())
    else {
        return Err(diagnostic(
            command,
            State::Error,
            format!("referenced prompt {} is not declared", command.prompt()),
            "prompt missing",
        ));
    };
    let Some(projection) = prompt
        .projections()
        .find(|projection| projection.agent == command.agent && projection.scope == command.scope)
    else {
        return Err(diagnostic(
            command,
            State::Error,
            "referenced prompt has no projection for this binding",
            "projection missing",
        ));
    };
    if projection.destination != expected_destination {
        return Err(diagnostic(
            command,
            State::Error,
            format!(
                "prompt projection destination {} does not match {}",
                projection.destination.display(),
                expected_destination.display()
            ),
            "projection destination incompatible",
        ));
    }
    Ok((prompt, projection))
}

fn unsupported(agent: Option<Agent>, scope: Option<Scope>, name: Option<&str>) -> Diagnostic {
    let subject = match (agent, scope, name) {
        (Some(agent), Some(scope), Some(name)) => format!("{agent} {scope} command {name}"),
        (Some(agent), Some(scope), None) => format!("{agent} {scope} commands"),
        (Some(agent), None, None) => format!("{agent} commands"),
        (None, Some(scope), None) => format!("{scope} command scope"),
        _ => "command bindings".to_owned(),
    };
    let group = match (agent, scope) {
        (Some(agent), Some(scope)) => format!("{agent} {scope} commands"),
        _ => "commands".to_owned(),
    };
    Diagnostic::new(
        "commands",
        State::Unsupported,
        format!("{subject} is not declared or has no stable command contract"),
    )
    .with_human(group, "capability · unsupported")
}

fn unsupported_binding(command: CommandBinding<'_>) -> Diagnostic {
    unsupported(
        Some(command.agent),
        Some(command.scope),
        Some(command.name()),
    )
}

fn broken(command: CommandBinding<'_>, failure: Failure) -> Diagnostic {
    diagnostic(command, failure.state, failure.message, failure.summary)
}

fn diagnostic(
    command: CommandBinding<'_>,
    state: State,
    message: impl Into<String>,
    summary: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        "commands",
        state,
        format!("{}: {}", subject(command), message.into()),
    )
    .with_human(
        format!("{} {} commands", command.agent, command.scope),
        format!("{} · {}", command.name(), summary.into()),
    )
}

fn subject(command: CommandBinding<'_>) -> String {
    let destination = capability::destination(command.agent, command.scope, command.name())
        .expect("supported bindings have a destination");
    format!(
        "managed {} {} command {} at {}",
        command.agent,
        command.scope,
        command.name(),
        destination.display()
    )
}
