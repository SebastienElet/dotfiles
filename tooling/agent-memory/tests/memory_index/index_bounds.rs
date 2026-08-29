use super::support::*;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

#[test]
fn a_legitimate_index_above_one_mibibyte_loads_fresh_repeatedly() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let terms = (0..20)
        .map(|number| format!("{number:02}{}", "x".repeat(98)))
        .collect::<Vec<_>>();
    let term_refs = terms.iter().map(String::as_str).collect::<Vec<_>>();
    let initial_statement = "Large legitimate index entry 0.";
    let initial_id = admit_user(
        &store,
        fixture.path(),
        initial_statement,
        &term_refs,
        &"s".repeat(160),
    );
    let initial_yaml =
        fs::read_to_string(root.join(format!("entries/user/{initial_id}.yaml"))).unwrap();
    let initial_index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    let template = initial_index["entries"][0].clone();
    let mut rows = Vec::new();
    let mut inventory = Vec::new();
    for number in 0..520 {
        let statement = format!("Large legitimate index entry {number}.");
        let id = memory_id(&statement);
        let path = format!("entries/user/{id}.yaml");
        let yaml = initial_yaml
            .replace(&initial_id, &id)
            .replace(initial_statement, &statement);
        let absolute = root.join(&path);
        fs::write(&absolute, yaml).unwrap();
        fs::set_permissions(&absolute, fs::Permissions::from_mode(0o600)).unwrap();
        let metadata = fs::metadata(&absolute).unwrap();
        let mut row = template.clone();
        row["id"] = id.into();
        row["path"] = path.clone().into();
        row["statement_tokens"] =
            serde_json::json!([number.to_string(), "entry", "index", "large", "legitimate"]);
        row["length"] = metadata.len().into();
        row["modified_ns"] = modified_ns(&metadata).into();
        rows.push(row);
        inventory.push(InventoryFixture {
            path,
            length: metadata.len(),
            modified_ns: modified_ns(&metadata),
        });
    }
    rows.sort_by(|left, right| left["path"].as_str().cmp(&right["path"].as_str()));
    inventory.sort_by(|left, right| left.path.cmp(&right.path));
    let digest = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&inventory).unwrap())
    );
    let document = serde_json::json!({
        "schema_version": 1,
        "inventory_digest": digest,
        "entries": rows,
        "diagnostics": { "user": [], "projects": {} },
    });
    let bytes = format!("{}\n", serde_json::to_string_pretty(&document).unwrap()).into_bytes();
    assert!(bytes.len() > 1024 * 1024);
    fs::write(root.join("index.json"), &bytes).unwrap();

    let first = Index::load_or_rebuild(&store).unwrap();
    let second = Index::load_or_rebuild(&store).unwrap();

    assert!(!first.rebuilt);
    assert!(!second.rebuilt);
    assert_eq!(fs::read(root.join("index.json")).unwrap(), bytes);
}

fn memory_id(statement: &str) -> String {
    let preimage = serde_json::to_vec(&(1_u8, "invariant", "user", statement)).unwrap();
    let digest = format!("{:x}", Sha256::digest(preimage));
    format!("mem_{}", &digest[..24])
}

fn modified_ns(metadata: &fs::Metadata) -> i64 {
    metadata.mtime() * 1_000_000_000 + metadata.mtime_nsec()
}

#[derive(Serialize)]
struct InventoryFixture {
    path: String,
    length: u64,
    modified_ns: i64,
}
