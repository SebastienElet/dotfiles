use crate::files::paths::{ancestor_within, canonical_within};
use std::fs;
use std::path::Path;

pub fn read(path: &Path, root: &Path) -> Result<Option<Vec<u8>>, &'static str> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return if ancestor_within(path, root) {
                Ok(None)
            } else {
                Err("parent resolves outside inspection root")
            };
        }
        Err(_) => return Err("metadata could not be read"),
        Ok(_) if canonical_within(path, root).is_none() => {
            return Err("path resolves outside inspection root");
        }
        Ok(_) => {}
    }
    fs::read(path)
        .map(Some)
        .map_err(|_| "file could not be read")
}

pub fn quote(value: &str) -> String {
    if !value.is_empty()
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.@/:".contains(&byte))
    {
        value.to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}
