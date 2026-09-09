use super::contracts::{BehavioralCase, Fixture, Prompt, Trigger, validate_path};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSource {
    pub path: String,
    pub heading: String,
    pub fingerprint: String,
}

#[derive(Clone, Debug)]
pub struct LoadedCase {
    pub definition: BehavioralCase,
    pub sources: Vec<ResolvedSource>,
    pub prompt: String,
    pub prompt_fingerprint: String,
    pub fixture: Fixture,
}

pub fn fingerprint(value: impl AsRef<[u8]>) -> String {
    format!("{:x}", Sha256::digest(value.as_ref()))
}

pub fn source_path(root: &Path, path: &str) -> Result<PathBuf, String> {
    validate_path(path)?;
    let root = root.canonicalize().map_err(|e| e.to_string())?;
    let canonical = root.join(path).canonicalize().map_err(|e| e.to_string())?;
    if !canonical.starts_with(&root) {
        return Err(format!("Escaping source: {path}"));
    }
    Ok(canonical)
}

pub fn read_source(root: &Path, path: &str) -> Result<String, String> {
    fs::read_to_string(source_path(root, path)?).map_err(|e| e.to_string())
}

pub fn section<'a>(markdown: &'a str, heading: &str) -> Result<&'a str, String> {
    let start = markdown
        .find(&format!("## {heading}\n"))
        .ok_or_else(|| format!("Missing section: {heading}"))?;
    let end = markdown[start + 1..]
        .find("\n## ")
        .map_or(markdown.len(), |i| start + 1 + i);
    Ok(&markdown[start..end])
}

pub fn load_trigger(root: &Path, path: &str) -> Result<Trigger, String> {
    let trigger: Trigger =
        serde_json::from_str(&read_source(root, path)?).map_err(|e| e.to_string())?;
    trigger.validate()?;
    let slug = Path::new(path)
        .parent()
        .and_then(Path::parent)
        .and_then(Path::file_name)
        .and_then(|s| s.to_str());
    if slug != Some(trigger.skill.as_str()) {
        return Err(format!("Skill slug mismatch: {path}"));
    }
    Ok(trigger)
}

pub fn load_cases(root: &Path) -> Result<Vec<LoadedCase>, String> {
    let definitions: Vec<BehavioralCase> =
        serde_json::from_str(&read_source(root, "harness/evals/cases.json")?)
            .map_err(|e| e.to_string())?;
    if definitions.is_empty() {
        return Err("Empty cases".into());
    }
    let mut ids = HashSet::new();
    definitions
        .into_iter()
        .map(|definition| {
            if !ids.insert(definition.id.clone()) {
                return Err("Duplicate case ID".into());
            }
            resolve_case(root, definition)
        })
        .collect()
}

pub fn resolve_case(root: &Path, definition: BehavioralCase) -> Result<LoadedCase, String> {
    definition.validate()?;
    let sources = definition
        .sources
        .iter()
        .map(|source| {
            let content = read_source(root, &source.path)?;
            section(&content, &source.heading)?;
            Ok(ResolvedSource {
                path: source.path.clone(),
                heading: source.heading.clone(),
                fingerprint: fingerprint(content),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let prompt = match &definition.prompt {
        Prompt::Inline { text } => text.clone(),
        Prompt::Trigger {
            trigger_file,
            query_index,
        } => load_trigger(root, trigger_file)?
            .queries
            .get(*query_index)
            .map(|q| q.query.clone())
            .unwrap_or_default(),
    };
    if prompt.is_empty() {
        return Err(format!("Unresolved prompt: {}", definition.id));
    }
    let fixture: Fixture = serde_json::from_str(&read_source(
        root,
        &format!("harness/evals/fixtures/{}.json", definition.fixture),
    )?)
    .map_err(|e| e.to_string())?;
    fixture.validate()?;
    if fixture.id != definition.fixture {
        return Err("Fixture identity mismatch".into());
    }
    Ok(LoadedCase {
        definition,
        sources,
        prompt_fingerprint: fingerprint(&prompt),
        prompt,
        fixture,
    })
}

#[cfg(test)]
#[path = "sources/tests.rs"]
mod tests;
