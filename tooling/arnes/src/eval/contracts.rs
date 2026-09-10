use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
#[path = "contracts/tests.rs"]
mod tests;

/// # Errors
/// Rejects empty, absolute, dot, parent, or non-ASCII path components.
pub fn validate_path(path: &str) -> Result<(), String> {
    if path.split('/').any(|part| {
        part.is_empty()
            || part == "."
            || part == ".."
            || !part
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
    }) {
        return Err(format!("Non-relative path: {path}"));
    }
    Ok(())
}

/// # Errors
/// Rejects an empty string.
pub fn nonempty(value: &str) -> Result<(), String> {
    if value.is_empty() {
        Err("Expected nonempty text".into())
    } else {
        Ok(())
    }
}

/// # Errors
/// Rejects fingerprints with the wrong length or characters outside lowercase hexadecimal.
pub fn hash(value: &str, length: usize) -> Result<(), String> {
    if value.len() != length
        || !value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("Invalid fingerprint".into());
    }
    Ok(())
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    pub skill: String,
    pub version: String,
    pub queries: Vec<Query>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Query {
    pub query: String,
    pub should_activate: bool,
    pub reason: String,
}
impl Trigger {
    /// # Errors
    /// Rejects an empty skill identifier or a query set missing either activation polarity.
    pub fn validate(&self) -> Result<(), String> {
        nonempty(&self.skill)?;
        if !self.queries.iter().any(|q| q.should_activate)
            || !self.queries.iter().any(|q| !q.should_activate)
        {
            return Err("Both activation polarities are required".into());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Oracle {
    StructuralV1,
    LiteralV1,
    KnownPathV1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tool {
    Cat,
    Rg,
    Fd,
    ColgrepSearch,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Observation {
    pub tool: Tool,
    pub args: Vec<String>,
    pub exit_code: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub path: String,
    pub heading: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum Prompt {
    Inline {
        text: String,
    },
    Trigger {
        #[serde(rename = "triggerFile")]
        trigger_file: String,
        #[serde(rename = "queryIndex")]
        query_index: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BehavioralCase {
    pub id: String,
    pub expected: String,
    pub sources: Vec<Source>,
    pub prompt: Prompt,
    pub fixture: String,
    pub oracle: Oracle,
    pub success: String,
    pub failure: String,
}

impl BehavioralCase {
    /// # Errors
    /// Rejects invalid case identifiers, empty expectations or sources, invalid prompts, or unknown fixtures.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.split('-').any(|p| {
            p.is_empty()
                || !p
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        }) {
            return Err("Invalid case ID".into());
        }
        for text in [&self.expected, &self.success, &self.failure] {
            nonempty(text)?;
        }
        if self.sources.is_empty() {
            return Err("Empty sources".into());
        }
        for source in &self.sources {
            validate_path(&source.path)?;
            nonempty(&source.heading)?;
        }
        match &self.prompt {
            Prompt::Inline { text } => nonempty(text)?,
            Prompt::Trigger {
                trigger_file,
                query_index,
            } => {
                validate_path(trigger_file)?;
                if *query_index > 9_007_199_254_740_991 {
                    return Err("Unsafe query index".into());
                }
            }
        }
        if self.fixture != "code-search-v1" {
            return Err("Unknown fixture".into());
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub id: String,
    pub files: BTreeMap<String, String>,
}

impl Fixture {
    /// # Errors
    /// Rejects unknown or empty fixtures, invalid relative paths, or empty file contents.
    pub fn validate(&self) -> Result<(), String> {
        if self.id != "code-search-v1" || self.files.is_empty() {
            return Err("Invalid fixture".into());
        }
        for (path, text) in &self.files {
            validate_path(path)?;
            nonempty(text)?;
        }
        Ok(())
    }
}
