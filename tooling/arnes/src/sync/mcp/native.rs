use super::read;
use crate::manifest::{Agent, McpRegistration, Scope};
use crate::sync::native_file::json::{self, Object};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(super) enum Document {
    Json(Object),
    Toml(toml::Table),
}

pub(super) struct Native {
    pub original: Option<read::Snapshot>,
    pub document: Document,
}

pub(super) fn path(agent: Agent, scope: Scope) -> PathBuf {
    PathBuf::from(match (agent, scope) {
        (Agent::Claude, Scope::User) => ".claude.json",
        (Agent::Claude, Scope::Project) => ".mcp.json",
        (Agent::Cursor, _) => ".cursor/mcp.json",
        (Agent::Codex, _) => ".codex/config.toml",
    })
}

impl Native {
    pub fn load(root: &Path, agent: Agent, scope: Scope) -> Result<Self, &'static str> {
        let original = read::optional(root, &path(agent, scope))?;
        let bytes = original
            .as_ref()
            .map_or(&[][..], |snapshot| snapshot.bytes.as_slice());
        let document = if agent == Agent::Codex {
            let text =
                std::str::from_utf8(bytes).map_err(|_| "native configuration is not UTF-8")?;
            Document::Toml(toml::from_str(text).map_err(|_| "native TOML is malformed")?)
        } else {
            Document::Json(json::parse(if original.is_none() { b"{}" } else { bytes })?)
        };
        Ok(Self { original, document })
    }

    pub fn entry(&self, name: &str) -> Result<Option<Vec<u8>>, &'static str> {
        match &self.document {
            Document::Json(document) => {
                let servers = json::child(document, "mcpServers")?;
                let Some(entry) = json::get::<Object>(&servers, name)? else {
                    return Ok(None);
                };
                validate_json(&entry)?;
                serde_json::to_vec(&entry)
                    .map(Some)
                    .map_err(|_| "native entry cannot be rendered")
            }
            Document::Toml(document) => {
                let Some(servers) = document.get("mcp_servers") else {
                    return Ok(None);
                };
                let servers = servers.as_table().ok_or("mcp_servers must be a table")?;
                let Some(entry) = servers.get(name) else {
                    return Ok(None);
                };
                let entry = entry.as_table().ok_or("MCP entry must be a table")?;
                if entry.contains_key("url") || entry.contains_key("type") {
                    return Err("MCP entry has unsupported transport fields");
                }
                toml::to_string(entry)
                    .map(|text| Some(text.into_bytes()))
                    .map_err(|_| "native entry cannot be rendered")
            }
        }
    }

    pub fn patch(&mut self, expected: McpRegistration<'_>) -> Result<(), &'static str> {
        match &mut self.document {
            Document::Json(document) => patch_json(document, expected),
            Document::Toml(document) => patch_toml(document, expected),
        }
    }

    pub fn render(&self) -> Result<Vec<u8>, &'static str> {
        match &self.document {
            Document::Json(document) => {
                serde_json::to_vec_pretty(document).map_err(|_| "native JSON cannot be rendered")
            }
            Document::Toml(document) => toml::to_string(document)
                .map(String::into_bytes)
                .map_err(|_| "native TOML cannot be rendered"),
        }
    }
}

fn validate_json(entry: &Object) -> Result<(), &'static str> {
    if entry.contains_key("url")
        || entry.contains_key("enabled")
        || json::get::<String>(entry, "type")?.is_some_and(|kind| kind != "stdio")
    {
        return Err("MCP entry has unsupported transport or enabled fields");
    }
    Ok(())
}

fn patch_json(document: &mut Object, expected: McpRegistration<'_>) -> Result<(), &'static str> {
    let mut servers = json::child(document, "mcpServers")?;
    let mut entry = json::get::<Object>(&servers, expected.name)?.unwrap_or_default();
    validate_json(&entry)?;
    json::put(&mut entry, "type", &"stdio")?;
    json::put(&mut entry, "command", &expected.command)?;
    json::put(&mut entry, "args", &expected.args)?;
    let environment = expected
        .environment
        .iter()
        .map(|name| {
            let value = match expected.agent {
                Agent::Cursor => format!("${{env:{name}}}"),
                Agent::Claude | Agent::Codex => format!("${{{name}}}"),
            };
            (name, value)
        })
        .collect::<BTreeMap<_, _>>();
    json::put(&mut entry, "env", &environment)?;
    json::put(&mut servers, expected.name, &entry)?;
    json::put(document, "mcpServers", &servers)
}

fn patch_toml(
    document: &mut toml::Table,
    expected: McpRegistration<'_>,
) -> Result<(), &'static str> {
    let servers = document
        .entry("mcp_servers")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or("mcp_servers must be a table")?;
    let entry = servers
        .entry(expected.name)
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or("MCP entry must be a table")?;
    entry.insert("command".to_owned(), expected.command.into());
    entry.insert("args".to_owned(), strings(expected.args));
    entry.insert("env_vars".to_owned(), strings(expected.environment));
    if let Some(enabled) = expected.enabled {
        entry.insert("enabled".to_owned(), enabled.into());
    }
    Ok(())
}

fn strings(values: &[String]) -> toml::Value {
    toml::Value::Array(values.iter().cloned().map(toml::Value::String).collect())
}
