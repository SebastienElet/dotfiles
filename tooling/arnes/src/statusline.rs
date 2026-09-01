pub mod configuration;

use crate::Roots;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest, Scope, Statusline};

pub fn diagnose(
    roots: &Roots,
    manifest: &Manifest,
    agent: Option<Agent>,
    scope: Option<Scope>,
) -> Vec<Diagnostic> {
    // Claude Code status lines execute shell commands (code.claude.com/docs/en/statusline), while Cursor publishes no persistent schema; only Codex tui.status_line is diagnosed statically.
    manifest
        .statuslines()
        .filter(|statusline| agent.is_none_or(|agent| agent == statusline.agent))
        .filter(|statusline| scope.is_none_or(|scope| scope == statusline.scope))
        .map(|statusline| diagnose_statusline(roots, statusline))
        .collect()
}

fn diagnose_statusline(roots: &Roots, statusline: Statusline<'_>) -> Diagnostic {
    let identity = format!("{} {}", statusline.agent, statusline.scope);
    match configuration::load(roots, statusline.scope) {
        Ok(Some(items)) if items == statusline.items => Diagnostic::new(
            "statusline",
            State::Healthy,
            format!("{identity}: status line matches manifest"),
        ),
        Ok(Some(_)) => Diagnostic::new(
            "statusline",
            State::Drift,
            format!("{identity}: ordered items differ"),
        ),
        Ok(None) => Diagnostic::new(
            "statusline",
            State::Drift,
            format!("{identity}: configuration is missing"),
        ),
        Err(error) => Diagnostic::new("statusline", State::Error, format!("{identity}: {error}")),
    }
}
