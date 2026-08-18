use serde::Deserialize;

#[derive(Deserialize)]
struct Metadata {
    description: Option<String>,
}

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
    let metadata: Metadata =
        serde_yaml_ng::from_str(frontmatter).map_err(|_| "frontmatter missing or malformed")?;
    match metadata.description.as_deref() {
        Some(description) if description == expected => Ok(()),
        Some(_) => Err("description differs from manifest"),
        None => Err("description is missing"),
    }
}
