use super::support::*;
use agent_memory::Index;
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::os::unix::fs::PermissionsExt;

#[test]
fn a_legitimate_index_above_one_mibibyte_loads_fresh_repeatedly()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
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
    )?;
    let initial_yaml = fs::read_to_string(root.join(format!("entries/user/{initial_id}.yaml")))?;
    let initial_index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
    let template = initial_index
        .pointer("/entries/0")
        .ok_or("missing index entries/0")?
        .clone();
    let mut rows = Vec::new();
    let mut inventory = Vec::new();
    for number in 0..520 {
        let statement = format!("Large legitimate index entry {number}.");
        let id = memory_id(&statement)?;
        let path = format!("entries/user/{id}.yaml");
        let yaml = initial_yaml
            .replace(&initial_id, &id)
            .replace(initial_statement, &statement);
        let absolute = root.join(&path);
        fs::write(&absolute, yaml)?;
        fs::set_permissions(&absolute, fs::Permissions::from_mode(0o600))?;
        let metadata = fs::metadata(&absolute)?;
        let mut row = template.clone();
        *row.get_mut("id").ok_or("missing index id")? = id.into();
        *row.get_mut("path").ok_or("missing index path")? = path.clone().into();
        *row.get_mut("statement_tokens")
            .ok_or("missing index statement_tokens")? =
            serde_json::json!([number.to_string(), "entry", "index", "large", "legitimate"]);
        *row.get_mut("length").ok_or("missing index length")? = metadata.len().into();
        *row.get_mut("modified_ns")
            .ok_or("missing index modified_ns")? = modified_ns(&metadata).into();
        rows.push(row);
        inventory.push(InventoryFixture {
            path,
            length: metadata.len(),
            modified_ns: modified_ns(&metadata),
        });
    }
    rows.sort_by(|left, right| {
        left.get("path")
            .and_then(serde_json::Value::as_str)
            .cmp(&right.get("path").and_then(serde_json::Value::as_str))
    });
    inventory.sort_by(|left, right| left.path.cmp(&right.path));
    let digest = format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(&inventory)?)
    );
    let document = serde_json::json!({
        "schema_version": 2,
        "inventory_digest": digest,
        "entries": rows,
        "diagnostics": { "user": [], "projects": {} },
    });
    let bytes = format!("{}\n", serde_json::to_string_pretty(&document)?).into_bytes();
    assert!(bytes.len() > 1024 * 1024);
    fs::write(root.join("index.json"), &bytes)?;

    let first = Index::load_or_rebuild(&store)?;
    let second = Index::load_or_rebuild(&store)?;

    assert!(!first.rebuilt);
    assert!(!second.rebuilt);
    assert_eq!(fs::read(root.join("index.json"))?, bytes);
    Ok(())
}

fn memory_id(statement: &str) -> Result<String, serde_json::Error> {
    let preimage = serde_json::to_vec(&(1_u8, "invariant", "user", statement))?;
    let digest = format!("{:x}", Sha256::digest(preimage));
    Ok(format!(
        "mem_{}",
        digest.chars().take(24).collect::<String>()
    ))
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
