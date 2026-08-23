use serde_yaml_ng::{Mapping, Value};

pub(super) fn validate(contents: &str, expected: &str) -> Result<(), &'static str> {
    let normalized = contents.replace("\r\n", "\n");
    let frontmatter = normalized
        .strip_prefix("---\n")
        .and_then(|contents| {
            contents
                .split_once("\n---\n")
                .map(|(frontmatter, _)| frontmatter)
        })
        .ok_or("frontmatter missing or malformed")?;
    let metadata: Mapping =
        serde_yaml_ng::from_str(frontmatter).map_err(|_| "frontmatter missing or malformed")?;
    match metadata.get("description") {
        Some(Value::String(description)) if description == expected => Ok(()),
        Some(Value::String(_)) => Err("description differs from manifest"),
        Some(_) => Err("description must be a string"),
        None => Err("description is missing"),
    }
}
