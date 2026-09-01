use super::support::*;
use std::fs;
use std::os::unix::fs::PermissionsExt;

#[test]
fn rebuilds_absent_corrupt_and_oversized_indexes_atomically_with_private_mode() {
    for replacement in [
        None,
        Some(b"{corrupt".to_vec()),
        Some(vec![b'x'; 1024 * 1024 + 1]),
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        let id = admit_user(
            &store,
            fixture.path(),
            "Index rebuild authority.",
            &["index rebuild"],
            "Established.",
        );
        let index_path = root.join("index.json");
        match replacement {
            Some(bytes) => fs::write(&index_path, bytes).unwrap(),
            None => fs::remove_file(&index_path).unwrap(),
        }

        let loaded = Index::load_or_rebuild(&store).unwrap();

        assert!(loaded.rebuilt);
        assert!(loaded.diagnostics.is_empty());
        assert_eq!(index_ids(&root), vec![id]);
        assert_eq!(
            fs::metadata(index_path).unwrap().permissions().mode() & 0o777,
            0o600
        );
    }
}

#[test]
fn rebuilds_when_yaml_is_added_modified_or_deleted() {
    for mutation in ["added", "modified", "deleted"] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        let first = admit_user(
            &store,
            fixture.path(),
            "First authority.",
            &["first authority"],
            "Established.",
        );
        let stale = fs::read(root.join("index.json")).unwrap();
        let second = admit_user(
            &store,
            fixture.path(),
            "Second authority.",
            &["second authority"],
            "Established.",
        );
        match mutation {
            "added" => fs::write(root.join("index.json"), stale).unwrap(),
            "modified" => {
                let yaml = root.join(format!("entries/user/{first}.yaml"));
                let mut bytes = fs::read(&yaml).unwrap();
                bytes.push(b'\n');
                fs::write(yaml, bytes).unwrap();
            }
            "deleted" => fs::remove_file(root.join(format!("entries/user/{second}.yaml"))).unwrap(),
            _ => unreachable!(),
        }

        let loaded = Index::load_or_rebuild(&store).unwrap();

        assert!(loaded.rebuilt, "{mutation}");
        let expected = match mutation {
            "added" | "modified" => 2,
            "deleted" => 1,
            _ => unreachable!(),
        };
        assert_eq!(index_ids(&root).len(), expected, "{mutation}");
    }
}

#[test]
fn omits_unreadable_future_invalid_and_terminal_entries_with_redacted_diagnostics() {
    for (mutation, expected_check) in [
        ("malformed", "malformed_yaml"),
        ("future", "unsupported_schema"),
        ("invalid", "invalid_kind_status"),
        ("identity-statement", "entry_identity_mismatch"),
        ("identity-kind", "entry_identity_mismatch"),
        ("identity-scope", "entry_identity_mismatch"),
        ("terminal", "status"),
    ] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        let statement = format!("Private statement for {mutation}.");
        let id = admit_user(
            &store,
            fixture.path(),
            &statement,
            &["private retrieval"],
            "Established.",
        );
        let path = root.join(format!("entries/user/{id}.yaml"));
        let bytes = String::from_utf8(fs::read(&path).unwrap()).unwrap();
        let changed = match mutation {
            "malformed" => "not: [valid".to_owned(),
            "future" => bytes.replacen("schema_version: 1", "schema_version: 2", 1),
            "invalid" => bytes.replacen("status: active", "status: achieved", 1),
            "identity-statement" => bytes.replacen(&statement, "Edited authority.", 1),
            "identity-kind" => bytes.replacen("kind: invariant", "kind: evidence", 1),
            "identity-scope" => bytes.replacen(
                "scope:\n  type: user",
                "scope:\n  type: project\n  key: project_0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
                1,
            ),
            "terminal" => format!(
                "{}transition:\n  from: active\n  to: invalidated\n  at: 2026-08-28T13:00:00Z\n  verdict: invalid\n  reason: Proof changed.\n",
                bytes.replacen("status: active", "status: invalidated", 1)
            ),
            _ => unreachable!(),
        };
        fs::write(path, changed).unwrap();

        let loaded = Index::load_or_rebuild(&store).unwrap();

        assert!(loaded.rebuilt);
        assert!(index_ids(&root).is_empty());
        assert_eq!(loaded.diagnostics.len(), 1);
        let diagnostic = &loaded.diagnostics[0];
        assert_eq!(diagnostic.entry_id, id);
        assert_eq!(diagnostic.check, expected_check);
        assert_eq!(diagnostic.effect, "omitted");
        let rendered = format!("{diagnostic:?}");
        assert!(!rendered.contains(&statement));
        assert!(!rendered.contains("private retrieval"));

        let repeated = Index::load_or_rebuild(&store).unwrap();
        assert!(!repeated.rebuilt);
        assert_eq!(repeated.diagnostics, loaded.diagnostics);
    }
}

#[test]
fn repeated_rebuilds_are_byte_identical_and_rows_have_the_closed_shape() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    admit_user(
        &store,
        fixture.path(),
        "Béta alpha beta.",
        &["memory", "agent"],
        &"é".repeat(161),
    );
    let index_path = root.join("index.json");
    fs::remove_file(&index_path).unwrap();
    Index::load_or_rebuild(&store).unwrap();
    let first = fs::read(&index_path).unwrap();
    fs::remove_file(&index_path).unwrap();
    Index::load_or_rebuild(&store).unwrap();
    let second = fs::read(&index_path).unwrap();

    assert_eq!(first, second);
    let value: serde_json::Value = serde_json::from_slice(&second).unwrap();
    assert_eq!(value["schema_version"], 2);
    let row = value["entries"][0].as_object().unwrap();
    let mut keys = row.keys().cloned().collect::<Vec<_>>();
    keys.sort();
    assert_eq!(
        keys,
        [
            "id",
            "kind",
            "length",
            "modified_ns",
            "path",
            "retrieval_terms",
            "scope",
            "statement_tokens",
            "status",
            "summary",
        ]
    );
    assert_eq!(
        row["statement_tokens"],
        serde_json::json!(["alpha", "beta"])
    );
    assert_eq!(row["summary"], "é".repeat(160));
    assert!(row.get("statement").is_none());
}

fn index_ids(root: &std::path::Path) -> Vec<String> {
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    index["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_owned())
        .collect()
}
