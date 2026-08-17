use super::model::{Skill, Topology};
use crate::skills::paths::canonical_within;
use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(super) struct Inspection {
    pub name: Option<String>,
    pub version: Option<String>,
    pub topology: Topology,
    pub detail: Option<String>,
    pub skills: Vec<Skill>,
}

pub(super) fn inspect(root: &Path, candidates: &[&str]) -> Inspection {
    let manifest = match manifest_path(root, candidates) {
        Ok(Some(manifest)) => manifest,
        Ok(None) => return failure(Topology::Broken, "plugin manifest is missing"),
        Err((topology, detail)) => return failure(topology, detail),
    };
    let contents = match fs::read_to_string(&manifest) {
        Ok(contents) => contents,
        Err(_) => return failure(Topology::Unreadable, "plugin manifest could not be read"),
    };
    let value: Value = match serde_json::from_str(&contents) {
        Ok(value) => value,
        Err(_) => return failure(Topology::Broken, "plugin manifest is invalid JSON"),
    };
    let name = value.get("name").and_then(Value::as_str).map(str::to_owned);
    let version = value
        .get("version")
        .and_then(Value::as_str)
        .map(str::to_owned);
    let paths = match skill_paths(root, &value) {
        Ok(paths) => paths,
        Err(detail) => return failure(Topology::Broken, detail),
    };
    let skills = match discover_skills(root, paths) {
        Ok(skills) => skills,
        Err(detail) => return failure(Topology::Unreadable, detail),
    };
    Inspection {
        name,
        version,
        topology: Topology::Healthy,
        detail: None,
        skills,
    }
}

fn manifest_path(
    root: &Path,
    candidates: &[&str],
) -> Result<Option<PathBuf>, (Topology, &'static str)> {
    for candidate in candidates.iter().map(|candidate| root.join(candidate)) {
        match fs::symlink_metadata(&candidate) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(_) => return Err((Topology::Unreadable, "plugin manifest metadata failed")),
            Ok(_) => {}
        }
        match fs::metadata(&candidate) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err((Topology::Broken, "plugin manifest link is dangling"));
            }
            Err(_) => return Err((Topology::Unreadable, "plugin manifest metadata failed")),
            Ok(metadata) if !metadata.is_file() => {
                return Err((Topology::Broken, "plugin manifest is not a file"));
            }
            Ok(_) => {}
        }
        if canonical_within(&candidate, root).is_none() {
            return Err((
                Topology::Broken,
                "plugin manifest resolves outside its root",
            ));
        }
        return Ok(Some(candidate));
    }
    Ok(None)
}

fn skill_paths(root: &Path, value: &Value) -> Result<Vec<PathBuf>, &'static str> {
    let Some(skills) = value.get("skills") else {
        let default = root.join("skills");
        return if fs::symlink_metadata(&default).is_ok() {
            Ok(vec![default])
        } else if fs::symlink_metadata(root.join("SKILL.md")).is_ok() {
            Ok(vec![root.to_owned()])
        } else {
            Ok(Vec::new())
        };
    };
    let values = match skills {
        Value::String(path) => vec![path.as_str()],
        Value::Array(paths) => paths
            .iter()
            .map(Value::as_str)
            .collect::<Option<Vec<_>>>()
            .ok_or("plugin skills paths must be strings")?,
        _ => return Err("plugin skills must be a path or path list"),
    };
    values
        .into_iter()
        .map(|path| {
            let relative = Path::new(path);
            if relative
                .components()
                .all(|component| matches!(component, Component::CurDir | Component::Normal(_)))
            {
                Ok(root.join(relative))
            } else {
                Err("plugin skills path escapes its root")
            }
        })
        .collect()
}

fn discover_skills(root: &Path, paths: Vec<PathBuf>) -> Result<Vec<Skill>, &'static str> {
    let mut discovered = Vec::new();
    let mut seen = HashSet::new();
    for path in paths {
        if !seen.insert(path.clone()) {
            continue;
        }
        if fs::symlink_metadata(&path).is_err() {
            discovered.push(Skill {
                slug: slug(&path),
                path,
                topology: Topology::Broken,
                detail: Some("declared skill path is missing".to_owned()),
            });
            continue;
        }
        if canonical_within(&path, root).is_none() {
            discovered.push(Skill {
                slug: slug(&path),
                path,
                topology: Topology::Broken,
                detail: Some("skills directory resolves outside its plugin".to_owned()),
            });
            continue;
        }
        if fs::symlink_metadata(path.join("SKILL.md")).is_ok() {
            discovered.push(classify_skill(root, path));
            continue;
        }
        let entries = fs::read_dir(&path).map_err(|_| "skills directory could not be read")?;
        for entry in entries {
            let entry = entry.map_err(|_| "skills directory could not be read")?;
            let kind = entry
                .file_type()
                .map_err(|_| "skill metadata could not be read")?;
            if kind.is_dir() || kind.is_symlink() {
                discovered.push(classify_skill(root, entry.path()));
            }
        }
    }
    discovered.sort_by(|left, right| {
        left.slug
            .cmp(&right.slug)
            .then_with(|| left.path.cmp(&right.path))
    });
    Ok(discovered)
}

fn classify_skill(root: &Path, path: PathBuf) -> Skill {
    let slug = slug(&path);
    let result = if fs::metadata(&path).is_err() {
        (Topology::Broken, Some("skill link is dangling"))
    } else if canonical_within(&path, root).is_none() {
        (Topology::Broken, Some("skill resolves outside its plugin"))
    } else if !fs::metadata(&path).is_ok_and(|metadata| metadata.is_dir()) {
        (Topology::Broken, Some("skill is not a directory"))
    } else {
        let skill_file = path.join("SKILL.md");
        if fs::metadata(&skill_file).is_err() {
            (Topology::Broken, Some("SKILL.md is missing"))
        } else if canonical_within(&skill_file, root).is_none() {
            (
                Topology::Broken,
                Some("SKILL.md resolves outside its plugin"),
            )
        } else if fs::metadata(skill_file).is_ok_and(|metadata| metadata.is_file()) {
            (Topology::Healthy, None)
        } else {
            (Topology::Broken, Some("SKILL.md is not a file"))
        }
    };
    Skill {
        slug,
        path,
        topology: result.0,
        detail: result.1.map(str::to_owned),
    }
}

fn slug(path: &Path) -> String {
    path.file_name().map_or_else(
        || "unknown".to_owned(),
        |name| name.to_string_lossy().into(),
    )
}

fn failure(topology: Topology, detail: impl Into<String>) -> Inspection {
    Inspection {
        name: None,
        version: None,
        topology,
        detail: Some(detail.into()),
        skills: Vec::new(),
    }
}
