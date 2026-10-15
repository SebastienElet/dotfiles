use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::inspection::{quote, read};
use crate::manifest::{Agent, Manifest, Scope};
use serde_json::Value;
use std::path::{Path, PathBuf};

pub(super) struct Registration {
    pub agent: Agent,
    pub scope: Scope,
    pub name: String,
    pub path: PathBuf,
    pub project: Option<PathBuf>,
    pub local: bool,
    pub command: Option<String>,
    pub args: Vec<String>,
}

impl Registration {
    pub fn declared(&self, manifest: &Manifest) -> bool {
        !self.local
            && manifest.mcp_registrations().any(|expected| {
                expected.agent == self.agent
                    && expected.scope == self.scope
                    && expected.name == self.name
            })
    }

    pub fn removal(&self) -> String {
        let name = quote(&self.name);
        match self.agent {
            Agent::Codex if self.scope == Scope::User => format!("codex mcp remove {name}"),
            Agent::Claude => {
                let scope = if self.local {
                    "local"
                } else {
                    match self.scope {
                        Scope::User => "user",
                        Scope::Project => "project",
                    }
                };
                let command = format!("claude mcp remove -s {scope} {name}");
                self.project.as_ref().map_or_else(
                    || command.clone(),
                    |project| format!("cd {} && {command}", quote(&project.to_string_lossy())),
                )
            }
            Agent::Cursor => format!(
                "Cursor Settings > Tools & MCP: remove {name} from {} (no native CLI removal)",
                self.path.display()
            ),
            Agent::Codex => format!(
                "remove mcp_servers.{name} from {} (codex mcp remove targets user config only)",
                self.path.display()
            ),
        }
    }
}

pub(super) fn load(
    roots: &Roots,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> (Vec<Registration>, Vec<Diagnostic>) {
    let mut registrations = Vec::new();
    let mut diagnostics = Vec::new();
    for candidate in [Agent::Codex, Agent::Claude, Agent::Cursor] {
        if agent.is_some_and(|agent| agent != candidate) {
            continue;
        }
        for candidate_scope in [Scope::User, Scope::Project] {
            if scope.is_some_and(|scope| scope != candidate_scope) {
                continue;
            }
            let root = match candidate_scope {
                Scope::User => roots.home(),
                Scope::Project => roots.repository(),
            };
            let relative = match candidate {
                Agent::Codex => ".codex/config.toml",
                Agent::Claude if candidate_scope == Scope::User => ".claude.json",
                Agent::Claude => ".mcp.json",
                Agent::Cursor => ".cursor/mcp.json",
            };
            let path = root.join(relative);
            let result = read(&path, root).and_then(|bytes| {
                let Some(bytes) = bytes else {
                    return Ok(());
                };
                if candidate == Agent::Codex {
                    codex(&bytes, candidate_scope, &path, &mut registrations)
                } else {
                    json(
                        &bytes,
                        candidate,
                        candidate_scope,
                        &path,
                        scope != Some(Scope::User),
                        &mut registrations,
                    )
                }
            });
            if let Err(error) = result {
                diagnostics.push(Diagnostic::new(
                    "mcp",
                    State::Error,
                    format!("{candidate} {candidate_scope}: {}: {error}", path.display()),
                ));
            }
        }
    }
    if scope == Some(Scope::Project) && agent.is_none_or(|agent| agent == Agent::Claude) {
        let path = roots.home().join(".claude.json");
        let result = read(&path, roots.home()).and_then(|bytes| {
            let Some(bytes) = bytes else {
                return Ok(());
            };
            let value = super::json::parse(&bytes).map_err(|_| "invalid MCP JSON")?;
            local_projects(&value, &path, &mut registrations)
        });
        if let Err(error) = result {
            diagnostics.push(Diagnostic::new(
                "mcp",
                State::Error,
                format!("{}: {error}", path.display()),
            ));
        }
    }
    if scope == Some(Scope::User) {
        registrations.retain(|entry| entry.scope == Scope::User);
    }
    (registrations, diagnostics)
}

pub(super) fn diagnose(manifest: &Manifest, registrations: &[Registration]) -> Vec<Diagnostic> {
    registrations
        .iter()
        .filter(|entry| !entry.declared(manifest))
        .map(|entry| {
            Diagnostic::new(
                "mcp",
                State::Drift,
                format!(
                    "{} {} {}: undeclared MCP in {}; proposed removal: {}",
                    entry.agent,
                    if entry.local {
                        "local".to_owned()
                    } else {
                        entry.scope.to_string()
                    },
                    entry.name,
                    entry.path.display(),
                    entry.removal()
                ),
            )
        })
        .collect()
}

fn json(
    bytes: &[u8],
    agent: Agent,
    scope: Scope,
    path: &Path,
    include_projects: bool,
    entries: &mut Vec<Registration>,
) -> Result<(), &'static str> {
    let value = super::json::parse(bytes).map_err(|_| "invalid MCP JSON")?;
    if !value.is_object() {
        return Err("MCP configuration must be an object");
    }
    servers(&value, agent, scope, path, None, entries)?;
    if agent == Agent::Claude && scope == Scope::User && include_projects {
        local_projects(&value, path, entries)?;
    }
    Ok(())
}

fn local_projects(
    value: &Value,
    path: &Path,
    entries: &mut Vec<Registration>,
) -> Result<(), &'static str> {
    let Some(projects) = value.get("projects") else {
        return Ok(());
    };
    for (project, value) in projects.as_object().ok_or("projects must be an object")? {
        if !value.is_object() {
            return Err("project must be an object");
        }
        servers(
            value,
            Agent::Claude,
            Scope::Project,
            path,
            Some(Path::new(project)),
            entries,
        )?;
    }
    Ok(())
}

fn servers(
    value: &Value,
    agent: Agent,
    scope: Scope,
    path: &Path,
    project: Option<&Path>,
    entries: &mut Vec<Registration>,
) -> Result<(), &'static str> {
    let Some(servers) = value.get("mcpServers") else {
        return Ok(());
    };
    for (name, value) in servers.as_object().ok_or("mcpServers must be an object")? {
        let entry = value.as_object().ok_or("MCP entry must be an object")?;
        let command = entry
            .get("command")
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or("MCP command must be a string")
            })
            .transpose()?;
        let args = entry
            .get("args")
            .map(|value| {
                value
                    .as_array()
                    .ok_or("MCP args must be an array")
                    .and_then(|args| {
                        args.iter()
                            .map(|value| {
                                value
                                    .as_str()
                                    .map(str::to_owned)
                                    .ok_or("MCP argument must be a string")
                            })
                            .collect()
                    })
            })
            .transpose()?
            .unwrap_or_default();
        entries.push(Registration {
            agent,
            scope,
            name: name.clone(),
            path: path.to_owned(),
            project: project.map(Path::to_owned).or_else(|| {
                (scope == Scope::Project).then(|| path.parent().unwrap_or(path).to_owned())
            }),
            local: project.is_some(),
            command,
            args,
        });
    }
    Ok(())
}

fn codex(
    bytes: &[u8],
    scope: Scope,
    path: &Path,
    entries: &mut Vec<Registration>,
) -> Result<(), &'static str> {
    let value: toml::Value =
        toml::from_str(std::str::from_utf8(bytes).map_err(|_| "MCP configuration is not UTF-8")?)
            .map_err(|_| "invalid MCP TOML")?;
    let Some(servers) = value.get("mcp_servers") else {
        return Ok(());
    };
    for (name, value) in servers.as_table().ok_or("mcp_servers must be a table")? {
        let entry = value.as_table().ok_or("MCP entry must be a table")?;
        let command = entry
            .get("command")
            .map(|value| {
                value
                    .as_str()
                    .map(str::to_owned)
                    .ok_or("MCP command must be a string")
            })
            .transpose()?;
        let args = entry
            .get("args")
            .map(|value| {
                value
                    .as_array()
                    .ok_or("MCP args must be an array")
                    .and_then(|args| {
                        args.iter()
                            .map(|value| {
                                value
                                    .as_str()
                                    .map(str::to_owned)
                                    .ok_or("MCP argument must be a string")
                            })
                            .collect()
                    })
            })
            .transpose()?
            .unwrap_or_default();
        entries.push(Registration {
            agent: Agent::Codex,
            scope,
            name: name.clone(),
            path: path.to_owned(),
            project: None,
            local: false,
            command,
            args,
        });
    }
    Ok(())
}
