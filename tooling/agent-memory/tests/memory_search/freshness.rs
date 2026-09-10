use super::support::{admit_project, memory_root, project_scope};
use agent_memory::{Index, SearchRequest, SearchSelection, Store, search};
use sha2::Digest;
use std::fs;

#[test]
fn a_fresh_search_uses_statement_tokens_from_the_index_without_parsing_yaml()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project = project_scope(fixture.path(), "project-a.git")?;
    let selected = admit_project(
        &store,
        fixture.path(),
        &project,
        "Alpha beta are indexed.",
        &["unrelated term"],
        "Established.",
    )?;
    let yaml = find_yaml(&root, &selected)?;
    let before = fs::read(&yaml)?;
    fs::write(&yaml, b"not valid yaml")?;
    let metadata = fs::metadata(&yaml)?;
    let mut index_value: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
    let row = index_value
        .get_mut("entries")
        .ok_or("missing index entries")?
        .as_array_mut()
        .ok_or("missing fixture value")?
        .iter_mut()
        .find(|row| row.get("id").and_then(serde_json::Value::as_str) == Some(selected.as_str()))
        .ok_or("missing fixture value")?;
    *row.get_mut("length").ok_or("missing index length")? = metadata.len().into();
    *row.get_mut("modified_ns")
        .ok_or("missing index modified_ns")? = modified_ns(&metadata).into();
    let inventory = index_value
        .get("entries")
        .ok_or("missing index entries")?
        .as_array()
        .ok_or("missing fixture value")?
        .iter()
        .map(|row| {
            Ok::<_, Box<dyn std::error::Error + Send + Sync>>(InventoryFixture {
                path: row
                    .get("path")
                    .ok_or("missing index path")?
                    .as_str()
                    .ok_or("missing fixture value")?,
                length: row
                    .get("length")
                    .ok_or("missing index length")?
                    .as_u64()
                    .ok_or("missing fixture value")?,
                modified_ns: row
                    .get("modified_ns")
                    .ok_or("missing index modified_ns")?
                    .as_i64()
                    .ok_or("missing fixture value")?,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    *index_value
        .get_mut("inventory_digest")
        .ok_or("missing index inventory_digest")? = format!(
        "sha256:{:x}",
        sha2::Sha256::digest(serde_json::to_vec(&inventory)?)
    )
    .into();
    fs::write(
        root.join("index.json"),
        format!("{}\n", serde_json::to_string_pretty(&index_value)?),
    )?;
    let loaded = Index::load_or_rebuild(&store)?;
    let selection = search(
        &loaded.index,
        SearchRequest {
            query: "alpha beta",
            project_key: project.key(),
            include_user: false,
            limit: 5,
        },
    );

    assert!(!loaded.rebuilt);
    assert_eq!(ids(&selection), vec![selected]);
    assert_ne!(fs::read(yaml)?, before);
    Ok(())
}

fn ids(selection: &SearchSelection) -> Vec<String> {
    selection
        .selected
        .iter()
        .map(|entry| entry.entry_id.clone())
        .collect()
}

fn find_yaml(root: &std::path::Path, id: &str) -> std::io::Result<std::path::PathBuf> {
    for entry in fs::read_dir(root.join("entries/project"))? {
        let path = entry?.path().join(format!("{id}.yaml"));
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(std::io::Error::other(format!("missing YAML for {id}")))
}

fn modified_ns(metadata: &fs::Metadata) -> i64 {
    use std::os::unix::fs::MetadataExt;
    metadata.mtime() * 1_000_000_000 + metadata.mtime_nsec()
}

#[derive(serde::Serialize)]
struct InventoryFixture<'a> {
    path: &'a str,
    length: u64,
    modified_ns: i64,
}
