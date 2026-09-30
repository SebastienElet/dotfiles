use crate::Roots;
use crate::commands::{binding, capability};
use crate::manifest::{Agent, Manifest, Prompt, PromptProjection, PromptRepresentation, Scope};
use crate::prompts::source;
use crate::sync::selection::LinkIntent;

use super::owned::Intent;

pub(super) struct Candidate<'a> {
    pub id: String,
    pub prompt: Prompt<'a>,
    pub projection: PromptProjection<'a>,
}

pub(super) fn prompts(manifest: &Manifest, agent: Agent, scope: Scope) -> Vec<Candidate<'_>> {
    manifest
        .prompts()
        .flat_map(|prompt| {
            prompt
                .projections()
                .filter(move |projection| projection.agent == agent && projection.scope == scope)
                .map(move |projection| Candidate {
                    id: prompt.id().to_owned(),
                    prompt,
                    projection,
                })
        })
        .collect()
}

pub(super) fn commands(
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Result<Vec<Candidate<'_>>, &'static str> {
    let mut candidates = Vec::new();
    for command in manifest
        .commands()
        .flat_map(crate::manifest::Command::bindings)
        .filter(|binding| binding.agent == agent && binding.scope == scope)
    {
        let prompt = manifest
            .prompts()
            .find(|prompt| prompt.id() == command.prompt())
            .ok_or("referenced prompt is not declared")?;
        let projection = prompt
            .projections()
            .find(|projection| projection.agent == agent && projection.scope == scope)
            .ok_or("referenced prompt has no projection for this command")?;
        candidates.push(Candidate {
            id: command.name().to_owned(),
            prompt,
            projection,
        });
    }
    Ok(candidates)
}

pub(super) fn contents(
    roots: &Roots,
    manifest: &Manifest,
    candidate: &Candidate<'_>,
) -> Result<String, &'static str> {
    if candidate.projection.representation == PromptRepresentation::Symlink {
        return Err("symlink prompt projections are unsupported");
    }
    let expected = source::validate(roots, candidate.prompt)
        .map_err(|_| "prompt source, includes or variables are invalid")?;
    let contents = match candidate.projection.representation {
        PromptRepresentation::File => expected.direct,
        PromptRepresentation::Rendered => expected.rendered,
        PromptRepresentation::Symlink => return Err("symlink prompt projections are unsupported"),
    };
    for command in manifest
        .commands()
        .flat_map(crate::manifest::Command::bindings)
        .filter(|binding| {
            binding.prompt() == candidate.prompt.id()
                && binding.agent == candidate.projection.agent
                && binding.scope == candidate.projection.scope
        })
    {
        let expected_path = capability::destination(command.agent, command.scope, command.name())
            .ok_or("command capability is unsupported")?;
        if expected_path != candidate.projection.destination {
            return Err("command name and prompt destination are incompatible");
        }
        binding::validate(&contents, command.description())?;
    }
    Ok(contents)
}

pub(super) fn intent(
    roots: &Roots,
    manifest: &Manifest,
    candidate: &Candidate<'_>,
    contents: String,
) -> Result<Intent, &'static str> {
    let root = match candidate.projection.scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    if candidate.projection.scope == Scope::User {
        LinkIntent::regular_file(
            roots,
            candidate.id.clone(),
            root.to_owned(),
            candidate.projection.destination.to_owned(),
            roots.repository().join(candidate.prompt.source()),
            roots.repository().to_owned(),
            Scope::User,
        )
        .map_err(|_| "user projection aliases canonical repository sources")?;
    }
    protected(roots, manifest, candidate, &contents)?;
    Ok(Intent {
        id: candidate.id.clone(),
        owner: format!("prompt:{}", candidate.prompt.id()),
        root: root.to_owned(),
        destination: candidate.projection.destination.to_owned(),
        contents,
    })
}

fn protected(
    roots: &Roots,
    manifest: &Manifest,
    candidate: &Candidate<'_>,
    contents: &str,
) -> Result<(), &'static str> {
    let root = match candidate.projection.scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let destination = candidate.projection.destination;
    if std::fs::read(root.join(destination)).is_ok_and(|bytes| bytes == contents.as_bytes()) {
        return Ok(());
    }
    let name = destination
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("publication filename is invalid")?;
    let receipt = destination.with_file_name(format!(".{name}.arnes.json"));
    let paths = crate::sync::sources::publication_paths(&[destination.to_owned(), receipt])?;
    crate::sync::sources::protect_mutations(roots, manifest, candidate.projection.scope, &paths)
}
