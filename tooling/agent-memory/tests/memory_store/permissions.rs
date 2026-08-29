use super::support::*;

#[test]
fn open_list_and_admit_repair_existing_entry_and_scope_modes() {
    let fixture = tempfile::tempdir().unwrap();
    let root = fixture.path().join("agent-memory");
    let store = Store::open(memory_root(&root)).unwrap();
    let common = fixture.path().join("common.git");
    fs::create_dir(&common).unwrap();
    let scope_runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    let project = resolve_project(fixture.path(), &scope_runner).unwrap();
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z").unwrap();
    let draft = draft_yaml(
        "project",
        "Private existing entry.",
        "private existing entry",
        "Established.",
        "user-decision",
        "decision:private-existing-entry",
    );
    let id = stored_id(store.admit(
        resolved(&draft, &context),
        Some(&project),
        &timestamp,
        &context,
    ));
    let directory = root.join(format!("entries/project/{}", project.key().as_str()));
    let yaml = directory.join(format!("{id}.yaml"));

    expose(&directory, &yaml);
    let reopened = Store::open(memory_root(&root)).unwrap();
    assert_private(&directory, &yaml);

    expose(&directory, &yaml);
    assert_eq!(reopened.list().unwrap().entries().len(), 1);
    assert_private(&directory, &yaml);

    expose(&directory, &yaml);
    match reopened.admit(
        resolved(&draft, &context),
        Some(&project),
        &timestamp,
        &context,
    ) {
        AdmissionResult::Duplicate { id: duplicate } => assert_eq!(duplicate.as_str(), id),
        result => panic!("unexpected admission result: {result:?}"),
    }
    assert_private(&directory, &yaml);
}

fn expose(directory: &Path, yaml: &Path) {
    fs::set_permissions(directory, fs::Permissions::from_mode(0o755)).unwrap();
    fs::set_permissions(yaml, fs::Permissions::from_mode(0o644)).unwrap();
}

fn assert_private(directory: &Path, yaml: &Path) {
    assert_eq!(private_mode(directory), 0o700);
    assert_eq!(private_mode(yaml), 0o600);
}
