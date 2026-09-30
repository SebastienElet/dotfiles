use super::native::{Document, Native};
use crate::Roots;
use crate::manifest::{Agent, McpRegistration, Scope};
use crate::sync::native_file::{json, scope_root};

pub(super) struct Preferences {
    pub native: Native,
    pub changed: bool,
    pub enabled: bool,
}

impl Preferences {
    pub fn prepare(
        roots: &Roots,
        expected: McpRegistration<'_>,
    ) -> Result<Option<Self>, &'static str> {
        if expected.agent != Agent::Claude || expected.scope != Scope::Project {
            return Ok(None);
        }
        let root = scope_root(roots, Scope::User)?;
        let mut native = Native::load(root, Agent::Claude, Scope::User)?;
        let Document::Json(document) = &mut native.document else {
            return Err("Claude preferences must be JSON");
        };
        let mut projects = json::child(document, "projects")?;
        let key = roots
            .repository()
            .to_str()
            .ok_or("project path must be UTF-8")?;
        let mut project = json::child(&projects, key)?;
        let mut disabled =
            json::get::<Vec<String>>(&project, "disabledMcpServers")?.unwrap_or_default();
        let enabled = !disabled.iter().any(|name| name == expected.name);
        let changed = expected.enabled.is_some_and(|value| value != enabled);
        if changed {
            if expected.enabled == Some(false) {
                disabled.push(expected.name.to_owned());
            } else {
                disabled.retain(|name| name != expected.name);
            }
            json::put(&mut project, "disabledMcpServers", &disabled)?;
            json::put(&mut projects, key, &project)?;
            json::put(document, "projects", &projects)?;
        }
        Ok(Some(Self {
            native,
            changed,
            enabled,
        }))
    }
}
