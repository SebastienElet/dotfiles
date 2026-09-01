use super::super::json;
use super::super::observed::{EnvironmentValue, ObservedConfiguration, ObservedRegistration};
use super::{ConfigurationError, read_optional};
use crate::Roots;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub(super) fn load(
    roots: &Roots,
    bytes: &[u8],
    claude: bool,
    project_state: bool,
    managed_names: &[&str],
) -> Result<Option<ObservedConfiguration>, ConfigurationError> {
    let value = json::parse(bytes).map_err(|error| {
        ConfigurationError::new(format!("MCP configuration is malformed: {error}"))
    })?;
    let root = object(&value, "MCP configuration")?;
    let Some(servers) = root.get("mcpServers") else {
        return Ok(Some(ObservedConfiguration::default()));
    };
    let disabled = if project_state {
        claude_disabled(roots)?
    } else {
        Vec::new()
    };
    let registrations = object(servers, "mcpServers")?
        .iter()
        .filter(|(name, _)| managed_names.contains(&name.as_str()))
        .map(|(name, value)| {
            registration(name, value, claude, disabled.contains(name))
                .map(|registration| (name.clone(), registration))
        })
        .collect::<Result<_, _>>()?;
    Ok(Some(ObservedConfiguration { registrations }))
}

fn registration(
    name: &str,
    value: &Value,
    claude: bool,
    disabled: bool,
) -> Result<ObservedRegistration, ConfigurationError> {
    let entry = object(value, name)?;
    let command = string_field(entry, name, "command")?.to_owned();
    let args = string_array(entry.get("args"), name, "args")?;
    let environment = environment_map(entry.get("env"), name)?;
    Ok(ObservedRegistration {
        command,
        args,
        environment,
        enabled: claude.then_some(!disabled),
    })
}

fn object<'a>(value: &'a Value, field: &str) -> Result<&'a Map<String, Value>, ConfigurationError> {
    value
        .as_object()
        .ok_or_else(|| ConfigurationError::new(format!("{field} must be an object")))
}

fn string_field<'a>(
    entry: &'a Map<String, Value>,
    name: &str,
    field: &str,
) -> Result<&'a str, ConfigurationError> {
    entry
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| ConfigurationError::new(format!("{name}.{field} must be a string")))
}

fn string_array(
    value: Option<&Value>,
    name: &str,
    field: &str,
) -> Result<Vec<String>, ConfigurationError> {
    let Some(value) = value else {
        return Ok(Vec::new());
    };
    value
        .as_array()
        .ok_or_else(|| {
            ConfigurationError::new(format!("{name}.{field} must be an array of strings"))
        })?
        .iter()
        .map(|value| {
            value.as_str().map(str::to_owned).ok_or_else(|| {
                ConfigurationError::new(format!("{name}.{field} must be an array of strings"))
            })
        })
        .collect()
}

fn environment_map(
    value: Option<&Value>,
    name: &str,
) -> Result<BTreeMap<String, EnvironmentValue>, ConfigurationError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    object(value, &format!("{name}.env"))?
        .iter()
        .map(|(key, value)| {
            let value = value.as_str().ok_or_else(|| {
                ConfigurationError::new(format!("{name}.env.{key} must be a string"))
            })?;
            Ok((key.clone(), environment_value(value)))
        })
        .collect()
}

fn environment_value(value: &str) -> EnvironmentValue {
    value
        .strip_prefix("${")
        .and_then(|value| value.strip_suffix('}'))
        .filter(|value| !value.is_empty())
        .map(|value| EnvironmentValue::Reference(value.to_owned()))
        .unwrap_or(EnvironmentValue::RedactedLiteral)
}

fn claude_disabled(roots: &Roots) -> Result<Vec<String>, ConfigurationError> {
    let Some(bytes) = read_optional(&roots.home().join(".claude.json"), roots.home())? else {
        return Ok(Vec::new());
    };
    let value = json::parse(&bytes).map_err(|error| {
        ConfigurationError::new(format!("MCP configuration is malformed: {error}"))
    })?;
    let root = object(&value, "MCP configuration")?;
    let Some(project) = root
        .get("projects")
        .and_then(Value::as_object)
        .and_then(|projects| projects.get(roots.repository().to_string_lossy().as_ref()))
    else {
        return Ok(Vec::new());
    };
    string_array(
        project.get("disabledMcpServers"),
        "project",
        "disabledMcpServers",
    )
}
