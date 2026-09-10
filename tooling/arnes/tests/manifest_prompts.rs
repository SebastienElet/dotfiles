#![cfg(test)]
use arnes::manifest::{self, Agent, PromptRepresentation, Scope};
use std::fs;
use std::path::Path;
fn fixture(name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/manifest")
            .join(name),
    )?)
}
fn error(name: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(match manifest::parse(&fixture(name)?) {
        Ok(_) => return Err(format!("{name} unexpectedly passed validation").into()),
        Err(error) => error.to_string(),
    })
}
#[test]
fn prompt_getters_preserve_manifest_order_and_topology()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = manifest::parse(&fixture("valid-prompts.yaml")?)?;
    let prompts = manifest.prompts().collect::<Vec<_>>();
    assert_eq!(
        prompts.iter().map(|prompt| prompt.id()).collect::<Vec<_>>(),
        ["deploy", "review"]
    );
    assert_eq!(
        (*(prompts).first().ok_or("missing fixture index 0")?).source(),
        Path::new("harness/prompts/deploy.md")
    );
    assert_eq!(
        (*(prompts).first().ok_or("missing fixture index 0")?)
            .includes()
            .collect::<Vec<_>>(),
        [Path::new("fragments/context.md"), Path::new("../shared.md"),]
    );
    assert_eq!(
        (*(prompts).first().ok_or("missing fixture index 0")?)
            .variables()
            .collect::<Vec<_>>(),
        ["environment", "_ticket"]
    );
    let projections = (*(prompts).first().ok_or("missing fixture index 0")?)
        .projections()
        .collect::<Vec<_>>();
    assert_eq!(projections.len(), 3);
    assert_eq!(
        (
            (projections)
                .first()
                .ok_or("missing fixture index 0")?
                .agent,
            (projections)
                .first()
                .ok_or("missing fixture index 0")?
                .scope,
            (projections)
                .first()
                .ok_or("missing fixture index 0")?
                .representation,
            (projections)
                .first()
                .ok_or("missing fixture index 0")?
                .destination,
        ),
        (
            Agent::Claude,
            Scope::User,
            PromptRepresentation::Symlink,
            Path::new(".claude/commands/deploy.md"),
        )
    );
    assert_eq!(
        (projections)
            .get(1)
            .ok_or("missing fixture index 1")?
            .representation,
        PromptRepresentation::File
    );
    assert_eq!(
        (projections)
            .get(2)
            .ok_or("missing fixture index 2")?
            .representation,
        PromptRepresentation::Rendered
    );
    Ok(())
}
#[test]
fn absent_prompts_remain_compatible_with_manifest_v1()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = manifest::parse(&fixture("valid.yaml")?)?;
    assert_eq!(manifest.prompts().count(), 0);
    Ok(())
}
#[test]
fn prompt_declarations_are_required_and_normalized()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    assert_eq!(
        error("missing-prompt-field.yaml")?,
        "prompts[0]: prompts[0]: missing field `variables` at line 4 column 5"
    );
    assert_eq!(
        error("prompt-resource.yaml")?,
        "resources[0].kind: prompts must use normalized top-level declarations"
    );
    Ok(())
}
#[test]
fn prompt_identities_and_sources_are_unique() -> Result<(), Box<dyn std::error::Error + Send + Sync>>
{
    let _: () = for (name, expected) in [
        (
            "empty-prompt-id.yaml",
            "prompts[0].id: identifier cannot be empty",
        ),
        (
            "duplicate-prompt-id.yaml",
            "prompts[1].id: duplicates prompts[0].id",
        ),
        (
            "duplicate-prompt-source.yaml",
            "prompts[1].source: duplicates prompts[0].source",
        ),
        (
            "invalid-prompt-source.yaml",
            "prompts[0].source.root: prompt sources must be repository-relative",
        ),
        (
            "invalid-prompt-source-path.yaml",
            "prompts[0].source.path: path must stay within its declared root",
        ),
    ] {
        assert_eq!(error(name)?, expected);
    };
    Ok(())
}
#[test]
fn prompt_includes_and_variables_are_normalized()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (name, expected) in [
        (
            "duplicate-prompt-include.yaml",
            "prompts[0].includes[1]: duplicates prompts[0].includes[0]",
        ),
        (
            "invalid-prompt-include.yaml",
            "prompts[0].includes[0]: path must stay within the repository",
        ),
        (
            "duplicate-prompt-variable.yaml",
            "prompts[0].variables[1]: duplicates prompts[0].variables[0]",
        ),
        (
            "invalid-prompt-variable.yaml",
            "prompts[0].variables[0]: must be an identifier without variable syntax",
        ),
    ] {
        assert_eq!(error(name)?, expected);
    };
    Ok(())
}
#[test]
fn prompt_projections_are_unambiguous() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (name, expected) in [
        (
            "duplicate-prompt-projection.yaml",
            "prompts[0].projections[1].agent: duplicates prompts[0].projections[0]",
        ),
        (
            "duplicate-prompt-destination.yaml",
            "prompts[1].projections[0].destination: duplicates prompts[0].projections[0].destination",
        ),
        (
            "prompt-resource-destination.yaml",
            "prompts[0].projections[0].destination: duplicates resources[0].destination",
        ),
        (
            "invalid-prompt-projection.yaml",
            "prompts[0].projections[0].agent: agent is not declared",
        ),
        (
            "undeclared-prompt-scope.yaml",
            "prompts[0].projections[0].scope: scope is not declared for this agent",
        ),
        (
            "wrong-prompt-destination-root.yaml",
            "prompts[0].projections[0].destination.root: destination root is incompatible with prompt projection scope",
        ),
        (
            "invalid-prompt-destination-path.yaml",
            "prompts[0].projections[0].destination.path: path must stay within its declared root",
        ),
        (
            "off-registry-prompt-destination.yaml",
            "prompts[0].projections[0].destination.path: destination must be a Markdown file inside the agent reusable-prompt registry",
        ),
        (
            "invalid-prompt-destination-extension.yaml",
            "prompts[0].projections[0].destination.path: destination must be a Markdown file inside the agent reusable-prompt registry",
        ),
    ] {
        assert_eq!(error(name)?, expected);
    }
    assert!(manifest::parse(&fixture("prompt-source-destination.yaml")?).is_ok());
    let rendered = fixture("prompt-source-destination.yaml")?
        .replace("representation: file", "representation: rendered");
    assert_eq!(
        manifest::parse(&rendered)
            .err()
            .ok_or("operation unexpectedly succeeded")?
            .to_string(),
        "prompts[0].projections[0].destination: source and destination must differ"
    );
    Ok(())
}
