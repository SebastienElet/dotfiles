use super::support::*;
use agent_memory::Index;
use std::fs;

#[test]
fn diagnostics_are_sorted_by_the_bytewise_id_check_effect_tuple_and_remain_redacted()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project = project_scope(fixture.path(), "project.git")?;
    let user_id = admit_user(
        &store,
        fixture.path(),
        "Private user diagnostic statement.",
        &["private user term"],
        "Private user proof.",
    )?;
    let mut ids = vec![user_id];
    for number in 0..8 {
        ids.push(admit_project(
            &store,
            fixture.path(),
            &project,
            &format!("Private project diagnostic statement {number}."),
            &["private project term"],
            "Private project proof.",
        )?);
    }
    for id in &ids {
        let path = find_yaml(&root, id)?;
        fs::write(path, b"not: [valid")?;
    }

    let loaded = Index::load_or_rebuild(&store)?;

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
    expected.sort_unstable();
    assert_eq!(actual, expected);
    let index: serde_json::Value = serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
    let diagnostics = index
        .pointer("/diagnostics/user")
        .ok_or("missing index diagnostics/user")?
        .as_array()
        .ok_or("missing fixture value")?
        .iter()
        .chain(
            index
                .pointer("/diagnostics/projects")
                .ok_or("missing index diagnostics/projects")?
                .as_object()
                .ok_or("missing fixture value")?
                .values()
                .map(|items| items.as_array().ok_or("missing fixture value"))
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .flatten(),
        );
    for diagnostic in diagnostics {
        let mut keys = diagnostic
            .as_object()
            .ok_or("missing fixture value")?
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
    Ok(())
}

#[test]
fn an_index_row_with_a_missing_or_unknown_field_is_rebuilt()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for mutation in ["missing", "unknown", "unnormalized", "extra-diagnostic"] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let store = Store::open(&memory_root(&root)?)?;
        admit_user(
            &store,
            fixture.path(),
            "Closed index row.",
            &["closed index"],
            "Established.",
        )?;
        let path = root.join("index.json");
        let mut index: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        match mutation {
            "missing" => {
                index
                    .pointer_mut("/entries/0")
                    .ok_or("missing index entries/0")?
                    .as_object_mut()
                    .ok_or("missing fixture value")?
                    .remove("statement_tokens");
            }
            "unknown" => {
                index
                    .pointer_mut("/entries/0")
                    .ok_or("missing index entries/0")?
                    .as_object_mut()
                    .ok_or("missing fixture value")?
                    .insert("statement".to_owned(), "must not persist".into());
            }
            "unnormalized" => {
                index
                    .pointer_mut("/entries/0")
                    .ok_or("missing index entries/0")?
                    .as_object_mut()
                    .ok_or("missing fixture value")?
                    .insert("statement_tokens".to_owned(), serde_json::json!(["BÉTA"]));
            }
            "extra-diagnostic" => {
                let id = index
                    .pointer("/entries/0/id")
                    .ok_or("missing index entries/0/id")?
                    .as_str()
                    .ok_or("missing fixture value")?
                    .to_owned();
                *index
                    .pointer_mut("/diagnostics/user")
                    .ok_or("missing index diagnostics/user")? = serde_json::json!([{
                    "entry_id": id,
                    "check": "status",
                    "effect": "omitted",
                }]);
            }
            _ => return Err("unexpected fixture variant".into()),
        }
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&index)?),
        )?;

        let loaded = Index::load_or_rebuild(&store)?;

        assert!(loaded.rebuilt, "{mutation}");
        let rebuilt = fs::read_to_string(path)?;
        assert!(!rebuilt.contains("must not persist"));
        assert!(!rebuilt.contains("\"statement\""));
    }
    Ok(())
}

#[test]
fn rebuilds_rows_with_retrieval_terms_outside_the_yaml_contract()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (mutation, terms) in [
        ("empty", serde_json::json!([])),
        ("too-long", serde_json::json!(["x".repeat(101)])),
        ("too-many", serde_json::json!(vec!["term"; 21])),
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let store = Store::open(&memory_root(&root)?)?;
        admit_user(
            &store,
            fixture.path(),
            "Retrieval term contract.",
            &["valid term"],
            "Established.",
        )?;
        let path = root.join("index.json");
        let mut index: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        *index
            .pointer_mut("/entries/0/retrieval_terms")
            .ok_or("missing index entries/0/retrieval_terms")? = terms;
        fs::write(
            &path,
            format!("{}\n", serde_json::to_string_pretty(&index)?),
        )?;

        let loaded = Index::load_or_rebuild(&store)?;

        assert!(loaded.rebuilt, "{mutation}");
        let rebuilt: serde_json::Value = serde_json::from_slice(&fs::read(&path)?)?;
        assert_eq!(
            *rebuilt
                .pointer("/entries/0/retrieval_terms")
                .ok_or("missing index entries/0/retrieval_terms")?,
            serde_json::json!(["valid term"]),
            "{mutation}"
        );
    }
    Ok(())
}

fn find_yaml(root: &std::path::Path, id: &str) -> std::io::Result<std::path::PathBuf> {
    let user = root.join(format!("entries/user/{id}.yaml"));
    if user.is_file() {
        return Ok(user);
    }
    for entry in fs::read_dir(root.join("entries/project"))? {
        let path = entry?.path().join(format!("{id}.yaml"));
        if path.is_file() {
            return Ok(path);
        }
    }
    Err(std::io::Error::other(format!("missing YAML for {id}")))
}
