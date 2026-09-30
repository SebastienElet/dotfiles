use super::read;
use crate::hooks::io::ConfigFile;
use crate::manifest::McpRegistration;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Deserialize, Serialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
struct Receipt {
    version: u8,
    root: PathBuf,
    agent: String,
    scope: String,
    name: String,
    digest: String,
}

pub(super) fn path(expected: McpRegistration<'_>) -> PathBuf {
    PathBuf::from(".arnes-mcp-ownership").join(format!(
        "{}-{}-{}.json",
        expected.agent, expected.scope, expected.name
    ))
}

fn receipt(
    root: &Path,
    expected: McpRegistration<'_>,
    entry: &[u8],
    enabled: Option<bool>,
) -> Result<Receipt, &'static str> {
    let mut digest = Sha256::new();
    digest.update(entry);
    digest.update(match enabled {
        None => b"unspecified".as_slice(),
        Some(true) => b"enabled",
        Some(false) => b"disabled",
    });
    Ok(Receipt {
        version: 1,
        root: root
            .canonicalize()
            .map_err(|_| "ownership root cannot be resolved")?,
        agent: expected.agent.to_string(),
        scope: expected.scope.to_string(),
        name: expected.name.to_owned(),
        digest: format!("{:x}", digest.finalize()),
    })
}

pub(super) fn matches(
    root: &Path,
    expected: McpRegistration<'_>,
    entry: &[u8],
    enabled: Option<bool>,
) -> bool {
    let Ok(Some(snapshot)) = read::optional(root, &path(expected)) else {
        return false;
    };
    let Ok(observed) = serde_json::from_slice::<Receipt>(&snapshot.bytes) else {
        return false;
    };
    receipt(root, expected, entry, enabled).is_ok_and(|expected| observed == expected)
}

pub(super) fn publish(
    root: &Path,
    expected: McpRegistration<'_>,
    entry: &[u8],
    enabled: Option<bool>,
) -> Result<(), &'static str> {
    let bytes = serde_json::to_vec(&receipt(root, expected, entry, enabled)?)
        .map_err(|_| "ownership receipt cannot be rendered")?;
    let file = ConfigFile::open_path(root, &path(expected))
        .map_err(|_| "ownership receipt cannot be safely opened")?;
    file.replace(&bytes)
        .map_err(|_| "ownership receipt publication failed")
}
