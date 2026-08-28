use super::support::*;
use std::fs;

#[test]
fn diagnostics_are_sorted_by_the_bytewise_id_check_effect_tuple_and_remain_redacted() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let project = project_scope(fixture.path(), "project.git");
    let user_id = admit_user(
        &store,
        fixture.path(),
        "Private user diagnostic statement.",
        &["private user term"],
        "Private user proof.",
    );
    let mut ids = vec![user_id];
    for number in 0..8 {
        ids.push(admit_project(
            &store,
            fixture.path(),
            &project,
            &format!("Private project diagnostic statement {number}."),
            &["private project term"],
            "Private project proof.",
        ));
    }
    for id in &ids {
        let path = find_yaml(&root, id);
        fs::write(path, b"not: [valid").unwrap();
    }

    let loaded = Index::load_or_rebuild(&store).unwrap();

    let actual = loaded
        .diagnostics
        .iter()
        .map(|diagnostic| {
            (
                diagnostic.entry_id.as_str(),
                diagnostic.check.as_str(),
                diagnostic.effect.as_str(),
            )
        })
        .collect::<Vec<_>>();
    let mut expected = ids
        .iter()
        .map(|id| (id.as_str(), "malformed_yaml", "omitted"))
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(actual, expected);
    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    for diagnostic in index["diagnostics"].as_array().unwrap() {
        let mut keys = diagnostic
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        keys.sort();
        assert_eq!(keys, ["check", "effect", "entry_id"]);
        let rendered = diagnostic.to_string();
        assert!(!rendered.contains("statement"));
        assert!(!rendered.contains("private project term"));
        assert!(!rendered.contains("private user term"));
        assert!(!rendered.contains("proof"));
        assert!(!rendered.contains("entries/"));
    }
}

#[test]
fn an_index_row_with_a_missing_or_unknown_field_is_rebuilt() {
    for mutation in ["missing", "unknown", "unnormalized", "extra-diagnostic"] {
        let fixture = tempfile::tempdir().unwrap();
        let root = fixture.path().join("agent-memory");
        let store = Store::open(memory_root(&root)).unwrap();
        admit_user(
            &store,
            fixture.path(),
            "Closed index row.",
            &["closed index"],
            "Established.",
        );
        let path = root.join("index.json");
        let mut index: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        match mutation {
            "missing" => {
                index["entries"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("statement_tokens");
            }
            "unknown" => {
                index["entries"][0]
                    .as_object_mut()
                    .unwrap()
                    .insert("statement".to_owned(), "must not persist".into());
            }
            "unnormalized" => {
                index["entries"][0]
                    .as_object_mut()
                    .unwrap()
                    .insert("statement_tokens".to_owned(), serde_json::json!(["BÉTA"]));
            }
            "extra-diagnostic" => {
                let id = index["entries"][0]["id"].as_str().unwrap().to_owned();
                index["diagnostics"] = serde_json::json!([{
                    "entry_id": id,
                    "check": "status",
                    "effect": "omitted",
                }]);
            }
            _ => unreachable!(),
        }
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&index).unwrap()),
        )
        .unwrap();

        let loaded = Index::load_or_rebuild(&store).unwrap();

        assert!(loaded.rebuilt, "{mutation}");
        let rebuilt = fs::read_to_string(path).unwrap();
        assert!(!rebuilt.contains("must not persist"));
        assert!(!rebuilt.contains("\"statement\""));
    }
}

fn find_yaml(root: &std::path::Path, id: &str) -> std::path::PathBuf {
    let user = root.join(format!("entries/user/{id}.yaml"));
    if user.is_file() {
        return user;
    }
    fs::read_dir(root.join("entries/project"))
        .unwrap()
        .map(|entry| entry.unwrap().path().join(format!("{id}.yaml")))
        .find(|path| path.is_file())
        .unwrap()
}
