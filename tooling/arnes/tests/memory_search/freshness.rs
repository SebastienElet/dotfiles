use super::support::{admit_project, memory_root, project_scope};
use arnes::memory::{Index, SearchRequest, SearchSelection, Store, search};
use sha2::Digest;
use std::fs;

#[test]
fn a_fresh_search_uses_statement_tokens_from_the_index_without_parsing_yaml() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project = project_scope(fixture.path(), "project-a.git");
    let selected = admit_project(
        &store,
        fixture.path(),
        &project,
        "Alpha beta are indexed.",
        &["unrelated term"],
        "Established.",
    );
    let yaml = find_yaml(&root, &selected);
    let before = fs::read(&yaml).unwrap();
    fs::write(&yaml, b"not valid yaml").unwrap();
    let metadata = fs::metadata(&yaml).unwrap();
    let mut index_value: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    let row = index_value["entries"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|row| row["id"] == selected)
        .unwrap();
    row["length"] = metadata.len().into();
    row["modified_ns"] = modified_ns(&metadata).into();
    let inventory = index_value["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| InventoryFixture {
            path: row["path"].as_str().unwrap(),
            length: row["length"].as_u64().unwrap(),
            modified_ns: row["modified_ns"].as_i64().unwrap(),
        })
        .collect::<Vec<_>>();
    index_value["inventory_digest"] = format!(
        "sha256:{:x}",
        sha2::Sha256::digest(serde_json::to_vec(&inventory).unwrap())
    )
    .into();
    fs::write(
        root.join("index.json"),
        format!("{}\n", serde_json::to_string_pretty(&index_value).unwrap()),
    )
    .unwrap();
    let loaded = Index::load_or_rebuild(&store).unwrap();
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
    assert_ne!(fs::read(yaml).unwrap(), before);
}

fn ids(selection: &SearchSelection) -> Vec<String> {
    selection
        .selected
        .iter()
        .map(|entry| entry.entry_id.clone())
        .collect()
}

fn find_yaml(root: &std::path::Path, id: &str) -> std::path::PathBuf {
    for directory in fs::read_dir(root.join("entries/project")).unwrap() {
        let candidate = directory.unwrap().path().join(format!("{id}.yaml"));
        if candidate.is_file() {
            return candidate;
        }
    }
    panic!("missing YAML for {id}")
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
