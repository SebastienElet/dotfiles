#![cfg(test)]
use arnes::manifest::{self, Agent, Scope};
fn input(commands: &str) -> String {
    format!(
        "version: 1
agents:
  - id: claude
    scopes: [user, project]
  - id: cursor
    scopes: [project]
  - id: codex
    scopes: [user, project]
prompts:
  - id: deploy
    source: {{ root: repository, path: harness/prompts/deploy.md }}
    includes: []
    variables: []
    projections: []
commands:{commands}
resources: []
"
    )
}
fn error(commands: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    Ok(manifest::parse(&input(commands))
        .err()
        .ok_or("operation unexpectedly succeeded")?
        .to_string())
}
#[test]
fn command_getters_preserve_command_and_binding_order()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = manifest::parse(&input(
        "
  - name: deploy
    description: Deploy safely
    prompt: deploy
    bindings:
      - { agent: claude, scope: user }
      - { agent: cursor, scope: project }",
    ))?;
    let commands = manifest.commands().collect::<Vec<_>>();
    assert_eq!(commands.len(), 1);
    assert_eq!(
        (*(commands).first().ok_or("missing fixture index 0")?).name(),
        "deploy"
    );
    assert_eq!(
        (*(commands).first().ok_or("missing fixture index 0")?).description(),
        "Deploy safely"
    );
    assert_eq!(
        (*(commands).first().ok_or("missing fixture index 0")?).prompt(),
        "deploy"
    );
    let bindings = (*(commands).first().ok_or("missing fixture index 0")?)
        .bindings()
        .collect::<Vec<_>>();
    assert_eq!(
        (
            (bindings).first().ok_or("missing fixture index 0")?.agent,
            (bindings).first().ok_or("missing fixture index 0")?.scope
        ),
        (Agent::Claude, Scope::User)
    );
    assert_eq!(
        (
            (bindings).get(1).ok_or("missing fixture index 1")?.agent,
            (bindings).get(1).ok_or("missing fixture index 1")?.scope
        ),
        (Agent::Cursor, Scope::Project)
    );
    assert_eq!(
        (*(bindings).get(1).ok_or("missing fixture index 1")?).name(),
        "deploy"
    );
    assert_eq!(
        (*(bindings).get(1).ok_or("missing fixture index 1")?).description(),
        "Deploy safely"
    );
    assert_eq!(
        (*(bindings).get(1).ok_or("missing fixture index 1")?).prompt(),
        "deploy"
    );
    Ok(())
}
#[test]
fn absent_commands_remain_compatible_with_manifest_v1()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let manifest = manifest::parse(&input(" []"))?;
    assert_eq!(manifest.commands().count(), 0);
    Ok(())
}
#[test]
fn command_fields_are_normalized() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let _: () = for (commands, expected) in [
        (
            "
    - name: ''
      description: Deploy safely
      prompt: deploy
      bindings: [{ agent: claude, scope: user }]",
            "commands[0].name: must be lowercase ASCII kebab-case",
        ),
        (
            "
    - name: Deploy
      description: Deploy safely
      prompt: deploy
      bindings: [{ agent: claude, scope: user }]",
            "commands[0].name: must be lowercase ASCII kebab-case",
        ),
        (
            "
    - name: deploy_now
      description: Deploy safely
      prompt: deploy
      bindings: [{ agent: claude, scope: user }]",
            "commands[0].name: must be lowercase ASCII kebab-case",
        ),
        (
            "
    - name: deploy--now
      description: Deploy safely
      prompt: deploy
      bindings: [{ agent: claude, scope: user }]",
            "commands[0].name: must be lowercase ASCII kebab-case",
        ),
        (
            "
    - name: deploy
      description: '  '
      prompt: deploy
      bindings: [{ agent: claude, scope: user }]",
            "commands[0].description: description cannot be blank",
        ),
        (
            "
    - name: deploy
      description: Deploy safely
      prompt: missing
      bindings: [{ agent: claude, scope: user }]",
            "commands[0].prompt: referenced prompt is not declared",
        ),
        (
            "
    - name: deploy
      description: Deploy safely
      prompt: deploy
      bindings: []",
            "commands[0].bindings: at least one binding is required",
        ),
        (
            "
    - name: deploy
      description: Deploy safely
      prompt: deploy
      bindings:
        - { agent: claude, scope: user }
        - { agent: claude, scope: user }",
            "commands[0].bindings[1].agent: duplicates commands[0].bindings[0]",
        ),
        (
            "
    - name: deploy
      description: One
      prompt: deploy
      bindings: [{ agent: claude, scope: user }]
    - name: deploy
      description: Two
      prompt: deploy
      bindings: [{ agent: claude, scope: user }]",
            "commands[1].bindings[0].agent: duplicates commands[0].bindings[0]",
        ),
    ] {
        assert_eq!(error(commands)?, expected);
    };
    Ok(())
}
#[test]
fn command_targets_must_be_declared() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command = "
  - name: deploy
    description: Deploy safely
    prompt: deploy
    bindings: [{ agent: cursor, scope: project }]";
    let without_cursor = input(command).replace(
        "  - id: cursor
    scopes: [project]
",
        "",
    );
    assert_eq!(
        manifest::parse(&without_cursor)
            .err()
            .ok_or("operation unexpectedly succeeded")?
            .to_string(),
        "commands[0].bindings[0].agent: agent is not declared"
    );
    let wrong_scope = command.replace("scope: project", "scope: user");
    assert_eq!(
        error(&wrong_scope)?,
        "commands[0].bindings[0].scope: scope is not declared for this agent"
    );
    Ok(())
}
#[test]
fn the_same_name_is_allowed_across_agents_and_scopes()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    manifest::parse(&input(
        "
  - name: deploy
    description: Deploy safely
    prompt: deploy
    bindings:
      - { agent: claude, scope: user }
      - { agent: claude, scope: project }
      - { agent: cursor, scope: project }",
    ))?;
    Ok(())
}
#[test]
fn legacy_command_resources_are_rejected() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let error = manifest::parse(&input(" []").replace(
        "resources: []",
        "resources:
  - id: deploy
    kind: commands
    agent: claude
    scope: project
    source: { root: repository, path: prompt.md }
    destination: { root: repository, path: .claude/commands/deploy.md }",
    ))
    .err()
    .ok_or("operation unexpectedly succeeded")?;
    assert_eq!(
        error.to_string(),
        "resources[0].kind: commands must use normalized top-level declarations"
    );
    Ok(())
}
#[test]
fn unknown_command_and_binding_fields_are_rejected()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command = "
  - name: deploy
    description: Deploy safely
    prompt: deploy
    extra: value
    bindings: [{ agent: claude, scope: user }]";
    assert!(error(command)?.contains("unknown field `extra`"));
    let binding = "
  - name: deploy
    description: Deploy safely
    prompt: deploy
    bindings: [{ agent: claude, scope: user, extra: value }]";
    assert!(error(binding)?.contains("unknown field `extra`"));
    Ok(())
}
