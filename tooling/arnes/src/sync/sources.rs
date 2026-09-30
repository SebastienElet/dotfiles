use crate::Roots;
use crate::files::includes::Resolver;
use crate::files::paths::planned_within;
use crate::manifest::{Manifest, Scope};
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn protect_mutations(
    roots: &Roots,
    manifest: &Manifest,
    scope: Scope,
    paths: &[PathBuf],
) -> Result<(), &'static str> {
    let root = match scope {
        Scope::User => roots.home(),
        Scope::Project => roots.repository(),
    };
    let reserved = reservations(roots, manifest)?;
    for path in paths {
        let path = root.join(path);
        let planned =
            planned_within(&path, root).ok_or("publication path cannot be resolved safely")?;
        if reserved.iter().any(|source| planned.starts_with(source)) {
            return Err("publication path aliases a canonical source");
        }
    }
    Ok(())
}

pub(super) fn publication_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>, &'static str> {
    let mut publication = Vec::new();
    for path in paths {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("publication filename is invalid")?;
        publication.push(path.clone());
        publication.push(path.with_file_name(format!(".{name}.lock")));
    }
    Ok(publication)
}

fn reservations(roots: &Roots, manifest: &Manifest) -> Result<Vec<PathBuf>, &'static str> {
    let mut reserved = Vec::new();
    for repository in [roots.repository(), roots.deployment_repository()] {
        for boundary in ["home", "harness", "harness/skills", ".agents/skills"] {
            reserve(&mut reserved, &repository.join(boundary), repository)?;
        }
    }
    reserved.push(planned_absolute(&roots.home().join(".arnes.yaml"))?);
    for resource in manifest.instruction_resources() {
        let root = source_root(roots, resource.scope);
        reserve_graph(&mut reserved, &root.join(resource.source), root)?;
    }
    for rule in manifest.rule_resources() {
        reserve_graph(
            &mut reserved,
            &roots.repository().join(rule.source),
            roots.repository(),
        )?;
    }
    for projection in manifest.skill_projections() {
        let root = source_root(roots, projection.scope);
        reserve(&mut reserved, &root.join(projection.source), root)?;
    }
    for prompt in manifest.prompts() {
        let source = roots.repository().join(prompt.source());
        reserve_graph(&mut reserved, &source, roots.repository())?;
        let parent = source.parent().ok_or("prompt source has no parent")?;
        let resolver = Resolver::new(roots.repository());
        for include in prompt.includes() {
            let include = include.to_str().ok_or("prompt include is not UTF-8")?;
            let included = resolver
                .resolve(parent, include)
                .map_err(|_| "prompt include cannot be confined")?;
            reserve_graph(&mut reserved, &included, roots.repository())?;
        }
    }
    for registration in manifest.mcp_registrations() {
        let command =
            crate::mcp::command::candidate(roots, registration.scope, registration.command)
                .map_err(|_| "MCP command source cannot be safely resolved")?;
        if let Some(command) = command {
            reserved.push(planned_absolute(&command)?);
        }
    }
    for command in crate::hooks::source_paths(roots, manifest) {
        reserved.push(planned_absolute(&command)?);
    }
    Ok(reserved)
}

fn source_root(roots: &Roots, scope: Scope) -> &Path {
    match scope {
        Scope::User => roots.deployment_repository(),
        Scope::Project => roots.repository(),
    }
}

fn reserve(paths: &mut Vec<PathBuf>, path: &Path, root: &Path) -> Result<(), &'static str> {
    paths.push(planned_within(path, root).ok_or("canonical source cannot be safely resolved")?);
    Ok(())
}

fn reserve_graph(paths: &mut Vec<PathBuf>, path: &Path, root: &Path) -> Result<(), &'static str> {
    reserve(paths, path, root)?;
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(_) => return Err("canonical source cannot be inspected"),
        Ok(_) => {}
    }
    let graph = Resolver::new(root)
        .walk(path)
        .map_err(|_| "canonical source include graph is invalid")?;
    for path in graph.paths() {
        reserve(paths, path, root)?;
    }
    Ok(())
}

fn planned_absolute(path: &Path) -> Result<PathBuf, &'static str> {
    if !path.is_absolute() {
        return Err("command candidate must be absolute");
    }
    planned_within(path, Path::new("/")).ok_or("command source cannot be safely resolved")
}
