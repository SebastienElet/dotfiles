use super::super::observed::{EnvironmentValue, ObservedConfiguration, ObservedRegistration};
use super::ConfigurationError;
use std::collections::BTreeMap;

pub(super) fn load(
    bytes: &[u8],
    managed_names: &[&str],
) -> Result<Option<ObservedConfiguration>, ConfigurationError> {
    let input = std::str::from_utf8(bytes)
        .map_err(|_| ConfigurationError::new("MCP configuration is not UTF-8"))?;
    let value = toml::from_str::<toml::Value>(input)
        .map_err(|_| ConfigurationError::new("MCP configuration is malformed"))?;
    let Some(servers) = value.get("mcp_servers") else {
        return Ok(Some(ObservedConfiguration::default()));
    };
    let servers = servers
        .as_table()
        .ok_or_else(|| ConfigurationError::new("mcp_servers must be a table"))?;
    let registrations = servers
        .iter()
        .filter(|(name, _)| managed_names.contains(&name.as_str()))
        .map(|(name, value)| {
            registration(name, value).map(|registration| (name.clone(), registration))
        })
        .collect::<Result<_, _>>()?;
    Ok(Some(ObservedConfiguration { registrations }))
}

fn registration(
    name: &str,
    value: &toml::Value,
) -> Result<ObservedRegistration, ConfigurationError> {
    let entry = value
        .as_table()
        .ok_or_else(|| ConfigurationError::new(format!("{name} must be a table")))?;
    let command = entry
        .get("command")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| ConfigurationError::new(format!("{name}.command must be a string")))?
        .to_owned();
    let args = string_array(entry.get("args"), name, "args")?;
    let mut environment = environment(entry.get("env"), name)?;
    for reference in string_array(entry.get("env_vars"), name, "env_vars")? {
        environment.insert(reference.clone(), EnvironmentValue::Reference(reference));
    }
    let enabled = entry
        .get("enabled")
        .map(|value| {
            value
                .as_bool()
                .ok_or_else(|| ConfigurationError::new(format!("{name}.enabled must be a boolean")))
        })
        .transpose()?
        .unwrap_or(true);
    Ok(ObservedRegistration {
        command,
        args,
        environment,
        enabled: Some(enabled),
    })
}

fn string_array(
    value: Option<&toml::Value>,
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

fn environment(
    value: Option<&toml::Value>,
    name: &str,
) -> Result<BTreeMap<String, EnvironmentValue>, ConfigurationError> {
    let Some(value) = value else {
        return Ok(BTreeMap::new());
    };
    value
        .as_table()
        .ok_or_else(|| ConfigurationError::new(format!("{name}.env must be a table")))?
        .iter()
        .map(|(key, value)| {
            value.as_str().ok_or_else(|| {
                ConfigurationError::new(format!("{name}.env.{key} must be a string"))
            })?;
            Ok((key.clone(), EnvironmentValue::RedactedLiteral))
        })
        .collect()
}
