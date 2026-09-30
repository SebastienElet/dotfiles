mod native;
mod ownership;
mod preferences;
mod read;

use super::native_file::scope_root;
use super::{SyncEntry, SyncState};
use crate::Roots;
use crate::hooks::io::ConfigFile;
use crate::manifest::{Agent, Manifest, McpRegistration, Scope};
use crate::mcp::{command, comparison, configuration};
use native::Native;
use preferences::Preferences;
use std::path::Path;

pub(super) fn synchronize(
    roots: &Roots,
    manifest: &Manifest,
    agent: Agent,
    scope: Scope,
) -> Vec<SyncEntry> {
    let selected = manifest
        .mcp_registrations()
        .filter(|registration| registration.agent == agent && registration.scope == scope)
        .collect::<Vec<_>>();
    if selected.is_empty() {
        return vec![SyncEntry::new(
            "mcp",
            SyncState::Empty,
            "no MCP registration is declared",
        )];
    }
    selected
        .into_iter()
        .map(|expected| synchronize_registration(roots, manifest, expected))
        .collect()
}

fn synchronize_registration(
    roots: &Roots,
    manifest: &Manifest,
    expected: McpRegistration<'_>,
) -> SyncEntry {
    let id = format!("{} {} {}", expected.agent, expected.scope, expected.name);
    if (expected.agent == Agent::Cursor && expected.enabled.is_some())
        || (expected.agent == Agent::Claude
            && expected.scope == Scope::User
            && expected.enabled == Some(false))
    {
        return SyncEntry::new(
            id,
            SyncState::Unsupported,
            "declared enabled state cannot be confirmed natively",
        );
    }
    match prepare(roots, manifest, expected) {
        Err(message) => SyncEntry::new(id, SyncState::Refused, message),
        Ok(None) => SyncEntry::new(
            id,
            SyncState::Current,
            "MCP registration is conforming without ownership adoption",
        ),
        Ok(Some(prepared)) => match prepared.publish(roots, expected) {
            Ok(()) => SyncEntry::new(id, SyncState::Applied, "MCP registration synchronized"),
            Err(message) => SyncEntry::new(id, SyncState::Failed, message),
        },
    }
}

struct Prepared {
    native: Native,
    preferences: Option<Preferences>,
    enabled: Option<bool>,
}

fn prepare(
    roots: &Roots,
    manifest: &Manifest,
    expected: McpRegistration<'_>,
) -> Result<Option<Prepared>, &'static str> {
    let root = scope_root(roots, expected.scope)?;
    if command::diagnose(roots, expected).is_some() {
        return Err("MCP command is unavailable or cannot be safely inspected");
    }
    let other_scope = match expected.scope {
        Scope::User => Scope::Project,
        Scope::Project => Scope::User,
    };
    let other_root = scope_root(roots, other_scope)?;
    if native::path(expected.agent, expected.scope) == native::path(expected.agent, other_scope)
        && root.canonicalize().ok() == other_root.canonicalize().ok()
    {
        return Err("MCP scope roots alias the same native destination");
    }
    let other = Native::load(other_root, expected.agent, other_scope)?;
    if other.entry(expected.name)?.is_some() {
        return Err("MCP registration collides with the other scope");
    }
    let mut native = Native::load(root, expected.agent, expected.scope)?;
    let old_entry = native.entry(expected.name)?;
    let preferences = Preferences::prepare(roots, expected)?;
    let observed = configuration::load(roots, expected.agent, expected.scope, &[expected.name])
        .map_err(|_| "MCP registration cannot be interpreted")?;
    let observed = observed
        .as_ref()
        .and_then(|configuration| configuration.registrations.get(expected.name));
    if observed.is_some_and(|observed| comparison::diagnose(expected, observed).is_empty()) {
        if Native::load(root, expected.agent, expected.scope)?.original != native.original
            || Native::load(other_root, expected.agent, other_scope)?.original != other.original
        {
            return Err("native configuration changed during inspection");
        }
        if let Some(preferences) = preferences.as_ref()
            && Native::load(roots.home(), Agent::Claude, Scope::User)?.original
                != preferences.native.original
        {
            return Err("Claude project preferences changed during inspection");
        }
        return Ok(None);
    }
    let old_enabled = observed.and_then(|registration| registration.enabled);
    if let Some(old_entry) = old_entry
        && !ownership::matches(root, expected, &old_entry, old_enabled)
    {
        return Err("divergent MCP registration has no matching Arnes ownership receipt");
    }
    let enabled = match expected.agent {
        Agent::Cursor => None,
        Agent::Claude => Some(expected.enabled.unwrap_or_else(|| {
            preferences
                .as_ref()
                .is_none_or(|preferences| preferences.enabled)
        })),
        Agent::Codex => Some(expected.enabled.or(old_enabled).unwrap_or(true)),
    };
    protect_publication(roots, manifest, expected, preferences.as_ref())?;
    native.patch(expected)?;
    Ok(Some(Prepared {
        native,
        preferences,
        enabled,
    }))
}

fn protect_publication(
    roots: &Roots,
    manifest: &Manifest,
    expected: McpRegistration<'_>,
    preferences: Option<&Preferences>,
) -> Result<(), &'static str> {
    let paths = super::sources::publication_paths(&[
        native::path(expected.agent, expected.scope),
        ownership::path(expected),
    ])?;
    super::sources::protect_mutations(roots, manifest, expected.scope, &paths)?;
    if preferences.is_some_and(|preferences| preferences.changed) {
        let paths = super::sources::publication_paths(&[native::path(Agent::Claude, Scope::User)])?;
        super::sources::protect_mutations(roots, manifest, Scope::User, &paths)?;
    }
    Ok(())
}

impl Prepared {
    fn publish(self, roots: &Roots, expected: McpRegistration<'_>) -> Result<(), &'static str> {
        let root = scope_root(roots, expected.scope)?;
        let identity = publish_native(root, expected.agent, expected.scope, &self.native)?;
        if let Some(preferences) = self.preferences
            && preferences.changed
        {
            let root = scope_root(roots, Scope::User)?;
            publish_native(root, Agent::Claude, Scope::User, &preferences.native)?;
        }
        let current = Native::load(root, expected.agent, expected.scope)?;
        if current.original.as_ref().map(|snapshot| snapshot.identity) != Some(identity) {
            return Err("native configuration changed before ownership publication");
        }
        let entry = self
            .native
            .entry(expected.name)?
            .ok_or("managed entry was not rendered")?;
        if current.entry(expected.name)?.as_ref() != Some(&entry) {
            return Err("native entry changed before ownership publication");
        }
        let observed = configuration::load(roots, expected.agent, expected.scope, &[expected.name])
            .map_err(|_| "published registration cannot be interpreted")?;
        if observed
            .and_then(|configuration| configuration.registrations.get(expected.name).cloned())
            .is_none_or(|observed| {
                observed.enabled != self.enabled
                    || !comparison::diagnose(expected, &observed).is_empty()
            })
        {
            return Err("published registration changed before ownership publication");
        }
        ownership::publish(root, expected, &entry, self.enabled)
    }
}

fn publish_native(
    root: &Path,
    agent: Agent,
    scope: Scope,
    native: &Native,
) -> Result<(u64, u64), &'static str> {
    let file = ConfigFile::open_path(root, &native::path(agent, scope))
        .map_err(|_| "native configuration cannot be safely opened")?;
    let original = native.original.as_ref();
    if file.content() != original.map(|snapshot| snapshot.bytes.as_slice())
        || file.identity() != original.map(|snapshot| snapshot.identity)
    {
        return Err("native configuration changed during synchronization");
    }
    file.replace_with_identity(&native.render()?)
        .map_err(|_| "native configuration publication failed")
}
