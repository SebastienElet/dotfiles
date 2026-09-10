use super::support::*;

#[test]
fn lists_yaml_authority_when_the_index_is_corrupt_or_larger_than_one_mibibyte()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft("YAML remains authority.", "yaml authority", "Established.");
    stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;
    let index = root.join("index.json");
    let oversized = serde_json::to_vec(&serde_json::json!({
        "schema_version": 2,
        "inventory_digest": format!("sha256:{}", "0".repeat(64)),
        "entries": [],
        "padding": "x".repeat(1024 * 1024),
    }))?;
    assert!(oversized.len() > 1024 * 1024);

    for bytes in [b"{corrupt".to_vec(), oversized] {
        fs::write(&index, bytes)?;

        let listing = store.list()?;

        assert_eq!(listing.entries().len(), 1);
        assert!(listing.index_rebuild_required());
    }
    Ok(())
}

#[test]
fn admission_omits_malformed_and_future_yaml_from_the_derived_index()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    for (mutation, expected_check) in [
        ("malformed", "malformed_yaml"),
        ("future", "unsupported_schema"),
    ] {
        let fixture = tempfile::tempdir()?;
        let root = fixture.path().join("agent-memory");
        let store = Store::open(&memory_root(&root)?)?;
        let runner = FakeProcessRunner::default();
        let context = SourceContext::new(fixture.path(), &runner, &runner);
        let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
        let existing_id = stored_id(store.admit(
            &resolved(
                &user_draft("Existing authority.", "existing authority", "Established."),
                &context,
            )?,
            None,
            &timestamp,
            &context,
        ))?;
        let existing_path = root.join(format!("entries/user/{existing_id}.yaml"));
        let bytes = fs::read_to_string(&existing_path)?;
        let changed = match mutation {
            "malformed" => "not: [valid".to_owned(),
            "future" => bytes.replacen("schema_version: 1", "schema_version: 2", 1),
            _ => return Err("unexpected fixture variant".into()),
        };
        fs::write(existing_path, changed)?;

        let added_id = stored_id(store.admit(
            &resolved(
                &user_draft("New authority.", "new authority", "Established."),
                &context,
            )?,
            None,
            &timestamp,
            &context,
        ))?;

        assert!(root.join(format!("entries/user/{added_id}.yaml")).is_file());
        let index: serde_json::Value = serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
        assert_eq!(
            index
                .get("entries")
                .ok_or("missing index entries")?
                .as_array()
                .ok_or("missing fixture value")?
                .len(),
            1,
            "{mutation}"
        );
        assert_eq!(
            *index
                .pointer("/entries/0/id")
                .ok_or("missing first index entry id")?,
            added_id,
            "{mutation}"
        );
        assert_eq!(
            *index
                .pointer("/diagnostics/user/0")
                .ok_or("missing user diagnostic")?,
            serde_json::json!({
                "entry_id": existing_id,
                "check": expected_check,
                "effect": "omitted",
            }),
            "{mutation}"
        );
    }
    Ok(())
}

#[test]
fn writes_private_user_and_project_entries_and_an_exact_minimal_index_row()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let summary = "é".repeat(161);
    let user = user_draft("User invariant.", "user invariant", &summary);
    let user_id = stored_id(store.admit(&resolved(&user, &context)?, None, &timestamp, &context))?;
    let common = fixture.path().join("common.git");
    fs::create_dir(&common)?;
    let scope_runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    let project = resolve_project(fixture.path(), &scope_runner)?;
    let project_draft = draft_yaml(
        "project",
        "Project invariant.",
        "project invariant",
        "Established.",
        "user-decision",
        "decision:project-invariant",
    );
    let project_id = stored_id(store.admit(
        &resolved(&project_draft, &context)?,
        Some(&project),
        &timestamp,
        &context,
    ))?;

    let user_path = root.join(format!("entries/user/{user_id}.yaml"));
    let project_directory = root.join(format!("entries/project/{}", project.key().as_str()));
    let project_path = project_directory.join(format!("{project_id}.yaml"));
    assert_eq!(private_mode(&user_path)?, 0o600);
    assert_eq!(private_mode(&project_directory)?, 0o700);
    assert_eq!(private_mode(&project_path)?, 0o600);
    let listing = store.list()?;
    assert_eq!(listing.entries().len(), 2);
    assert!(!listing.index_rebuild_required());
    assert_eq!(
        store
            .load(&user_id)?
            .ok_or("missing fixture value")?
            .id()
            .as_str(),
        user_id
    );
    assert_eq!(
        store
            .load(&project_id)?
            .ok_or("missing fixture value")?
            .id()
            .as_str(),
        project_id
    );

    let index: serde_json::Value = serde_json::from_slice(&fs::read(root.join("index.json"))?)?;
    assert_eq!(
        *index
            .get("schema_version")
            .ok_or("missing index schema version")?,
        2
    );
    assert!(
        index
            .get("inventory_digest")
            .ok_or("missing inventory digest")?
            .as_str()
            .ok_or("missing fixture value")?
            .strip_prefix("sha256:")
            .is_some_and(|digest| digest.len() == 64)
    );
    let rows = index
        .get("entries")
        .ok_or("missing index entries")?
        .as_array()
        .ok_or("missing fixture value")?;
    assert_eq!(rows.len(), 2);
    let user_row = rows
        .iter()
        .find(|row| row.get("id").and_then(serde_json::Value::as_str) == Some(user_id.as_str()))
        .ok_or("missing fixture value")?;
    assert_user_index_row(user_row, &user_id, &user_path)?;
    Ok(())
}

fn assert_user_index_row(
    user_row: &serde_json::Value,
    user_id: &str,
    user_path: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut keys = user_row
        .as_object()
        .ok_or("missing fixture value")?
        .keys()
        .cloned()
        .collect::<Vec<_>>();
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
    assert_eq!(*user_row.get("kind").ok_or("missing kind")?, "invariant");
    assert_eq!(*user_row.get("status").ok_or("missing status")?, "active");
    assert_eq!(
        *user_row
            .pointer("/scope/type")
            .ok_or("missing scope type")?,
        "user"
    );
    assert_eq!(
        *user_row
            .get("retrieval_terms")
            .ok_or("missing retrieval_terms")?,
        serde_json::json!(["user invariant"])
    );
    assert_eq!(
        *user_row.get("summary").ok_or("missing summary")?,
        "é".repeat(160)
    );
    assert_eq!(
        *user_row
            .get("statement_tokens")
            .ok_or("missing statement_tokens")?,
        serde_json::json!(["invariant", "user"])
    );
    assert_eq!(
        *user_row.get("path").ok_or("missing path")?,
        format!("entries/user/{user_id}.yaml")
    );
    assert_eq!(
        *user_row.get("length").ok_or("missing length")?,
        fs::metadata(user_path)?.len()
    );
    assert!(
        user_row
            .get("modified_ns")
            .ok_or("missing modified_ns")?
            .as_i64()
            .is_some()
    );
    Ok(())
}
