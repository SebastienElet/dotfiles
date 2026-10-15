use super::inventory::Registration;
use crate::diagnostic::{Diagnostic, State};
use crate::manifest::{Agent, Manifest};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::Command;

mod snapshot;
use snapshot::{Process, parse};

struct Group<'a> {
    agent: Agent,
    session: u32,
    name: String,
    members: Vec<&'a Process>,
}

pub(super) fn diagnose(
    manifest: &Manifest,
    registrations: &[Registration],
    agent: Option<Agent>,
) -> Vec<Diagnostic> {
    let result = Command::new("ps")
        .args(["-ax", "-o", "pid=,ppid=,rss=,etime=,command="])
        .output()
        .map_err(|_| "ps could not be executed")
        .and_then(|output| {
            if !output.status.success() {
                return Err("ps failed; process inventory unavailable");
            }
            let text = std::str::from_utf8(&output.stdout).map_err(|_| "ps output is not UTF-8")?;
            parse(text)
        });
    let processes = match result {
        Ok(processes) => processes,
        Err(error) => return vec![Diagnostic::new("mcp", State::Error, error)],
    };
    summarize(manifest, registrations, &processes, agent)
}

fn summarize(
    manifest: &Manifest,
    registrations: &[Registration],
    processes: &BTreeMap<u32, Process>,
    agent: Option<Agent>,
) -> Vec<Diagnostic> {
    let launches = processes
        .values()
        .filter_map(|process| {
            let (owner, _) = owner(process, processes)?;
            identify(process, owner, manifest, registrations).map(|name| (process.pid, name))
        })
        .collect::<BTreeMap<_, _>>();
    let mut groups = BTreeMap::new();
    for process in processes.values() {
        let Some((owner, session)) = owner(process, processes) else {
            continue;
        };
        if agent.is_some_and(|agent| agent != owner) {
            continue;
        }
        let Some(name) = launch_name(process, processes, &launches, session) else {
            continue;
        };
        groups
            .entry((owner.to_string(), session, name.to_owned()))
            .or_insert_with(|| Group {
                agent: owner,
                session,
                name: name.to_owned(),
                members: Vec::new(),
            })
            .members
            .push(process);
    }
    groups
        .into_values()
        .map(|group| diagnose_group(manifest, registrations, &group))
        .collect()
}

fn launch_name<'a>(
    process: &Process,
    processes: &BTreeMap<u32, Process>,
    launches: &'a BTreeMap<u32, String>,
    session: u32,
) -> Option<&'a str> {
    let mut current = process;
    let mut name = None;
    while current.pid != session {
        if let Some(launch) = launches.get(&current.pid) {
            name = Some(launch.as_str());
        }
        current = processes.get(&current.parent)?;
    }
    name
}

fn diagnose_group(
    manifest: &Manifest,
    registrations: &[Registration],
    group: &Group<'_>,
) -> Diagnostic {
    let expected = manifest
        .mcp_registrations()
        .filter(|expected| expected.agent == group.agent && expected.name == group.name)
        .collect::<Vec<_>>();
    let ids = group
        .members
        .iter()
        .map(|process| process.pid)
        .collect::<BTreeSet<_>>();
    let instances = group
        .members
        .iter()
        .filter(|process| !ids.contains(&process.parent))
        .collect::<Vec<_>>();
    let stale = instances.iter().any(|process| {
        !expected
            .iter()
            .any(|expected| launch_matches(expected.command, expected.args, &process.command))
    });
    let ram = group.members.iter().map(|process| process.rss).sum::<u64>();
    let oldest = group
        .members
        .iter()
        .map(|process| process.age)
        .max()
        .unwrap_or_default();
    let removal = registrations
        .iter()
        .find(|entry| entry.agent == group.agent && entry.name == group.name)
        .map_or_else(
            || {
                "no matching native registration; inspect the owning session before removal"
                    .to_owned()
            },
            Registration::removal,
        );
    let status = if stale {
        "command differs from declared configuration or MCP is undeclared"
    } else {
        "command matches declared configuration"
    };
    Diagnostic::new(
        "mcp",
        if stale { State::Drift } else { State::Healthy },
        format!(
            "{} MCP {} session={}: instances={} processes={} RAM={ram} KiB oldest={oldest}s scope=unavailable; {status} (advisory heuristic from ps ancestry and command signatures; argv redacted); proposed removal: {removal}; restart the owning agent session to release live processes",
            group.agent,
            group.name,
            group.session,
            instances.len(),
            group.members.len()
        ),
    )
}

fn owner(process: &Process, processes: &BTreeMap<u32, Process>) -> Option<(Agent, u32)> {
    let mut visited = BTreeSet::from([process.pid]);
    let mut parent = process.parent;
    let mut owner = None;
    while let Some(process) = processes.get(&parent) {
        if !visited.insert(process.pid) {
            return None;
        }
        if owner.is_none() {
            owner = match program(&process.command) {
                "codex" => Some((Agent::Codex, process.pid)),
                "claude" => Some((Agent::Claude, process.pid)),
                "cursor-agent" | "Cursor" => Some((Agent::Cursor, process.pid)),
                _ => None,
            };
        }
        parent = process.parent;
    }
    owner
}

fn identify(
    process: &Process,
    owner: Agent,
    manifest: &Manifest,
    registrations: &[Registration],
) -> Option<String> {
    if matches!(
        program(&process.command),
        "sh" | "bash" | "zsh" | "fish" | "ps" | "arnes" | "codex" | "claude"
    ) {
        return None;
    }
    let candidates = registrations
        .iter()
        .map(|entry| {
            (
                entry.agent,
                entry.name.as_str(),
                entry.command.as_deref(),
                entry.args.as_slice(),
            )
        })
        .chain(
            manifest
                .mcp_registrations()
                .map(|entry| (entry.agent, entry.name, Some(entry.command), entry.args)),
        );
    let ranked = candidates
        .map(|(agent, name, executable, args)| {
            (
                (
                    agent == owner,
                    signature(&process.command, name, executable, args),
                ),
                name,
            )
        })
        .filter(|((_, score), _)| *score > 0)
        .collect::<Vec<_>>();
    if let Some(best) = ranked.iter().map(|(rank, _)| *rank).max() {
        let names = ranked
            .iter()
            .filter(|(rank, _)| *rank == best)
            .map(|(_, name)| *name)
            .collect::<BTreeSet<_>>();
        return Some(if names.len() == 1 {
            names.first().copied().unwrap_or_default().to_owned()
        } else {
            "unidentified-mcp".to_owned()
        });
    }
    None
}

fn signature(command: &str, name: &str, executable: Option<&str>, args: &[String]) -> u8 {
    if executable.is_some_and(|executable| launch_matches(executable, args, command)) {
        return 4;
    }
    let executable_matches = executable.is_some_and(|executable| {
        let expected = program(executable);
        !runtime(expected) && expected == program(command)
    });
    let observed = package_target(program(command), command.split_whitespace().skip(1));
    if observed.is_some_and(|target| target.contains("mcp") && contains_name(target, name))
        || (executable_matches
            && command
                .split_whitespace()
                .any(|token| contains_name(token, name)))
    {
        return 3;
    }
    let expected = executable.and_then(|executable| {
        package_target(program(executable), args.iter().map(String::as_str))
    });
    if observed
        .zip(expected)
        .is_some_and(|(observed, expected)| observed == expected)
    {
        return 2;
    }
    u8::from(executable_matches)
}

fn package_target<'a>(
    executable: &str,
    mut arguments: impl Iterator<Item = &'a str>,
) -> Option<&'a str> {
    match executable {
        "npm" if matches!(arguments.next(), Some("exec" | "x")) => {}
        "npx" => {}
        _ => return None,
    }
    let target = arguments.find(|argument| !matches!(*argument, "-y" | "--yes" | "--no" | "--"))?;
    if target.starts_with('-') {
        return None;
    }
    let package = match target.rsplit_once('@').filter(|(name, _)| !name.is_empty()) {
        Some((name, version)) => {
            if version.is_empty()
                || !version.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric()
                        || matches!(byte, b'.' | b'-' | b'+' | b'^' | b'~' | b'*')
                })
            {
                return None;
            }
            name
        }
        None => target,
    };
    let parts = package.strip_prefix('@').map_or_else(
        || vec![package],
        |scoped| scoped.split('/').collect::<Vec<_>>(),
    );
    if (package.starts_with('@') && parts.len() != 2)
        || parts.iter().any(|part| {
            part.is_empty()
                || part.starts_with('.')
                || !part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        })
    {
        return None;
    }
    Some(package)
}

fn runtime(program: &str) -> bool {
    matches!(
        program,
        "node" | "npx" | "npm" | "uv" | "uvx" | "python" | "python3" | "bun" | "docker"
    )
}

fn contains_name(token: &str, name: &str) -> bool {
    !name.is_empty()
        && token.match_indices(name).any(|(offset, _)| {
            token
                .get(..offset)
                .and_then(|prefix| prefix.chars().next_back())
                .is_none_or(|character| !character.is_ascii_alphanumeric())
                && token
                    .get(offset + name.len()..)
                    .and_then(|suffix| suffix.chars().next())
                    .is_none_or(|character| !character.is_ascii_alphanumeric())
        })
}

fn program(command: &str) -> &str {
    command
        .split_whitespace()
        .next()
        .and_then(|executable| Path::new(executable).file_name())
        .and_then(|name| name.to_str())
        .unwrap_or_default()
}

// ps flattens argv; wrapper and quoting comparisons are advisory, not an execution boundary.
fn launch_matches(command: &str, args: &[String], observed: &str) -> bool {
    let mut tokens = observed.split_whitespace();
    let Some(executable) = tokens.next() else {
        return false;
    };
    let executable_matches = if command.contains('/') {
        command == executable
    } else {
        command == program(executable)
    };
    executable_matches && tokens.eq(args.iter().map(String::as_str))
}
