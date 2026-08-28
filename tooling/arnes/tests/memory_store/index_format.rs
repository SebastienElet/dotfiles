use super::support::*;

#[test]
fn writes_private_user_and_project_entries_and_an_exact_minimal_index_row() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let git = FakeProcessRunner::default();
    let curl = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &git, &curl);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let summary = "é".repeat(161);
    let user = user_draft("User invariant.", "user invariant", &summary);
    let user_id = stored_id(store.admit(resolved(&user, &context), None, &timestamp, &context));
    let common = fixture.path().join("common.git");
    fs::create_dir(&common).unwrap();
    let scope_runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    let project = resolve_project(fixture.path(), &scope_runner).unwrap();
    let project_draft = draft_yaml(
        "project",
        "Project invariant.",
        "project invariant",
        "Established.",
        "user-decision",
        "decision:project-invariant",
    );
    let project_id = stored_id(store.admit(
        resolved(&project_draft, &context),
        Some(&project),
        &timestamp,
        &context,
    ));

    let user_path = root.join(format!("entries/user/{user_id}.yaml"));
    let project_directory = root.join(format!("entries/project/{}", project.key().as_str()));
    let project_path = project_directory.join(format!("{project_id}.yaml"));
    assert_eq!(private_mode(&user_path), 0o600);
    assert_eq!(private_mode(&project_directory), 0o700);
    assert_eq!(private_mode(&project_path), 0o600);
    let listing = store.list().unwrap();
    assert_eq!(listing.entries().len(), 2);
    assert!(!listing.index_rebuild_required());
    assert_eq!(
        store.load(&user_id).unwrap().unwrap().id().as_str(),
        user_id
    );
    assert_eq!(
        store.load(&project_id).unwrap().unwrap().id().as_str(),
        project_id
    );

    let index: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("index.json")).unwrap()).unwrap();
    assert_eq!(index["schema_version"], 1);
    assert!(
        index["inventory_digest"]
            .as_str()
            .unwrap()
            .strip_prefix("sha256:")
            .is_some_and(|digest| digest.len() == 64)
    );
    let rows = index["entries"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    let user_row = rows.iter().find(|row| row["id"] == user_id).unwrap();
    let mut keys = user_row
        .as_object()
        .unwrap()
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
            "status",
            "summary",
        ]
    );
    assert_eq!(user_row["kind"], "invariant");
    assert_eq!(user_row["status"], "active");
    assert_eq!(user_row["scope"]["type"], "user");
    assert_eq!(
        user_row["retrieval_terms"],
        serde_json::json!(["user invariant"])
    );
    assert_eq!(user_row["summary"], "é".repeat(160));
    assert_eq!(user_row["path"], format!("entries/user/{user_id}.yaml"));
    assert_eq!(user_row["length"], fs::metadata(user_path).unwrap().len());
    assert!(user_row["modified_ns"].as_i64().is_some());
}
