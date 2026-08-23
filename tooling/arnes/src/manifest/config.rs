use super::{Agent, AgentDeclaration, ManifestError, Scope};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserConfig {
    pub model: String,
    pub effort: Option<String>,
    pub context_window: Option<u64>,
    pub auto_compact_window: Option<u64>,
    pub max_mode: Option<bool>,
}

pub(super) fn validate(
    agents: &[AgentDeclaration],
    agent_index: usize,
) -> Result<(), ManifestError> {
    let declaration = &agents[agent_index];
    let Some(config) = &declaration.user_config else {
        return Ok(());
    };
    let field = |name: &str| format!("agents[{agent_index}].user_config.{name}");
    if !declaration.scopes.contains(&Scope::User) {
        return Err(ManifestError::new(
            format!("agents[{agent_index}].user_config"),
            "requires the user scope",
        ));
    }
    if config.model.is_empty() {
        return Err(ManifestError::new(field("model"), "cannot be empty"));
    }
    validate_effort(config, declaration.id, &field)?;
    validate_capabilities(config, declaration.id, &field)
}

fn validate_effort(
    config: &UserConfig,
    agent: Agent,
    field: &impl Fn(&str) -> String,
) -> Result<(), ManifestError> {
    let Some(effort) = config.effort.as_deref() else {
        return Ok(());
    };
    if agent == Agent::Cursor {
        return Err(ManifestError::new(
            field("effort"),
            "cursor does not expose a persistent effort setting",
        ));
    }
    if matches!(effort, "low" | "medium" | "high" | "xhigh") {
        Ok(())
    } else {
        Err(ManifestError::new(
            field("effort"),
            "expected low, medium, high, or xhigh",
        ))
    }
}

fn validate_capabilities(
    config: &UserConfig,
    agent: Agent,
    field: &impl Fn(&str) -> String,
) -> Result<(), ManifestError> {
    if config.context_window == Some(0) {
        return Err(ManifestError::new(
            field("context_window"),
            "must be greater than zero",
        ));
    }
    if config.auto_compact_window == Some(0) {
        return Err(ManifestError::new(
            field("auto_compact_window"),
            "must be greater than zero",
        ));
    }
    if agent == Agent::Claude
        && config
            .auto_compact_window
            .is_some_and(|window| !(100_000..=1_000_000).contains(&window))
    {
        return Err(ManifestError::new(
            field("auto_compact_window"),
            "must be between 100000 and 1000000",
        ));
    }
    if agent == Agent::Codex
        && config
            .context_window
            .zip(config.auto_compact_window)
            .is_some_and(|(context, compact)| compact >= context)
    {
        return Err(ManifestError::new(
            field("auto_compact_window"),
            "must be smaller than context_window",
        ));
    }
    match agent {
        Agent::Codex if config.max_mode.is_some() => Err(ManifestError::new(
            field("max_mode"),
            "codex does not expose max mode",
        )),
        Agent::Claude if config.context_window.is_some() || config.max_mode.is_some() => {
            let name = if config.context_window.is_some() {
                "context_window"
            } else {
                "max_mode"
            };
            Err(ManifestError::new(
                field(name),
                "claude does not expose this persistent setting",
            ))
        }
        Agent::Cursor
            if config.context_window.is_some() || config.auto_compact_window.is_some() =>
        {
            let name = if config.context_window.is_some() {
                "context_window"
            } else {
                "auto_compact_window"
            };
            Err(ManifestError::new(
                field(name),
                "cursor does not expose this persistent setting",
            ))
        }
        _ => Ok(()),
    }
}
