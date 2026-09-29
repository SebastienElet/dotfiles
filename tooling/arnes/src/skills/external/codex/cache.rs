use crate::files::paths::canonical_within;
use crate::skills::external::manifest;
use crate::skills::external::model::{Exposure, Plugin, Topology};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(super) fn resolve(home: &Path, id: String, exposure: Exposure) -> Plugin {
    let Some((name, marketplace)) = identity(&id) else {
        return plugin(
            id,
            exposure,
            None,
            Topology::Unknown,
            "plugin identifier is not name@marketplace; its cache directory cannot be located",
        );
    };
    let cache = home.join(".codex/plugins/cache");
    let directory = cache.join(&marketplace).join(&name);
    let artifacts = match cached_artifacts(&cache, &directory) {
        Ok(artifacts) => artifacts,
        Err((topology, detail)) => return plugin(id, exposure, Some(directory), topology, detail),
    };
    match artifacts.as_slice() {
        [] => plugin(
            id,
            exposure,
            Some(directory),
            Topology::Missing,
            "no artifact is installed in the Codex plugin cache",
        ),
        [artifact] => inspect_artifact(id, &name, exposure, artifact),
        several => plugin(
            id,
            exposure,
            Some(directory),
            Topology::Unknown,
            format!(
                "several cached artifacts ({}); the active one is not observable without the Codex resolver",
                artifact_names(several).join(", "),
            ),
        ),
    }
}

fn identity(id: &str) -> Option<(String, String)> {
    let (name, marketplace) = id.split_once('@')?;
    (single_segment(name) && single_segment(marketplace))
        .then(|| (name.to_owned(), marketplace.to_owned()))
}

fn single_segment(value: &str) -> bool {
    let mut components = Path::new(value).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn cached_artifacts(
    cache: &Path,
    directory: &Path,
) -> Result<Vec<PathBuf>, (Topology, &'static str)> {
    match fs::symlink_metadata(directory) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(_) => {
            return Err((
                Topology::Unreadable,
                "plugin cache directory metadata failed",
            ));
        }
        Ok(_) => {}
    }
    let canonical_directory = canonical_within(directory, cache).ok_or((
        Topology::Broken,
        "plugin cache directory resolves outside the Codex plugin cache",
    ))?;
    let entries = fs::read_dir(directory).map_err(|_| {
        (
            Topology::Unreadable,
            "plugin cache directory could not be read",
        )
    })?;
    let mut artifacts = BTreeSet::new();
    for entry in entries {
        let entry = entry.map_err(|_| {
            (
                Topology::Unreadable,
                "plugin cache directory could not be read",
            )
        })?;
        if entry.file_name().to_string_lossy().starts_with('.') {
            continue;
        }
        let artifact = canonical_artifact(&entry.path(), directory, &canonical_directory)?;
        artifacts.extend(artifact);
    }
    Ok(artifacts.into_iter().collect())
}

fn canonical_artifact(
    entry: &Path,
    directory: &Path,
    canonical_directory: &Path,
) -> Result<Option<PathBuf>, (Topology, &'static str)> {
    if fs::metadata(entry).is_err() {
        return Err((Topology::Broken, "cached artifact link is dangling"));
    }
    let canonical = canonical_within(entry, directory).ok_or((
        Topology::Broken,
        "cached artifact resolves outside its plugin cache directory",
    ))?;
    if !canonical.is_dir() {
        return Ok(None);
    }
    if canonical.parent() != Some(canonical_directory) {
        return Err((
            Topology::Broken,
            "cached artifact aliases a path nested inside its plugin cache directory",
        ));
    }
    Ok(Some(canonical))
}

fn inspect_artifact(id: String, name: &str, exposure: Exposure, path: &Path) -> Plugin {
    let artifact = path
        .file_name()
        .map(|name| name.to_string_lossy().into_owned());
    let inspected = manifest::inspect(path, &[".codex-plugin/plugin.json"]);
    let (topology, detail) =
        if inspected.topology == Topology::Healthy && inspected.name.as_deref() != Some(name) {
            (
                Topology::Broken,
                Some("plugin manifest name does not match its identifier".to_owned()),
            )
        } else {
            (inspected.topology, inspected.detail)
        };
    Plugin {
        id,
        artifact,
        version: inspected.version,
        path: Some(path.to_owned()),
        exposure,
        topology,
        detail,
        skills: if topology == Topology::Healthy {
            inspected.skills
        } else {
            Vec::new()
        },
    }
}

fn artifact_names(artifacts: &[PathBuf]) -> Vec<String> {
    artifacts
        .iter()
        .filter_map(|path| path.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .collect()
}

fn plugin(
    id: String,
    exposure: Exposure,
    path: Option<PathBuf>,
    topology: Topology,
    detail: impl Into<String>,
) -> Plugin {
    Plugin {
        id,
        artifact: None,
        version: None,
        path,
        exposure,
        topology,
        detail: Some(detail.into()),
        skills: Vec::new(),
    }
}
