use super::support::*;
use agent_memory::{
    Clock, Index, OracleEnvironment, RetrievalContext, RetrievalRequest, SearchRequest,
    SourceResolution, SourceResolver, UtcTimestamp, retrieve, search,
};

#[test]
fn open_repairs_existing_entry_and_scope_modes()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let common = fixture.path().join("common.git");
    fs::create_dir(&common)?;
    let scope_runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    let project = resolve_project(fixture.path(), &scope_runner)?;
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = draft_yaml(
        "project",
        "Private existing entry.",
        "private existing entry",
        "Established.",
        "user-decision",
        "decision:private-existing-entry",
    );
    let id = stored_id(store.admit(
        &resolved(&draft, &context)?,
        Some(&project),
        &timestamp,
        &context,
    ))?;
    let directory = root.join(format!("entries/project/{}", project.key().as_str()));
    let yaml = directory.join(format!("{id}.yaml"));

    expose(&directory, &yaml)?;
    let reopened = Store::open(&memory_root(&root)?)?;
    assert_private(&directory, &yaml)?;
    assert_eq!(reopened.list()?.entries().len(), 1);
    Ok(())
}

fn expose(directory: &Path, yaml: &Path) -> std::io::Result<()> {
    fs::set_permissions(directory, fs::Permissions::from_mode(0o755))?;
    fs::set_permissions(yaml, fs::Permissions::from_mode(0o644))?;
    Ok(())
}

fn assert_private(directory: &Path, yaml: &Path) -> std::io::Result<()> {
    assert_eq!(private_mode(directory)?, 0o700);
    assert_eq!(private_mode(yaml)?, 0o600);
    Ok(())
}

#[test]
fn mutable_list_repairs_nested_directory_modes_after_open()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project = project_scope(fixture.path())?;
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = draft_yaml(
        "project",
        "Mutable list permissions.",
        "mutable list permissions",
        "Established.",
        "user-decision",
        "decision:mutable-list-permissions",
    );
    stored_id(store.admit(
        &resolved(&draft, &context)?,
        Some(&project),
        &timestamp,
        &context,
    ))?;
    let directories = [
        root.join("entries"),
        root.join("entries/project"),
        root.join(format!("entries/project/{}", project.key().as_str())),
    ];
    for directory in &directories {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o755))?;
    }

    assert_eq!(store.list()?.entries().len(), 1);

    for directory in directories {
        assert_eq!(private_mode(&directory)?, 0o700);
    }
    Ok(())
}

#[test]
fn mutable_admission_repairs_nested_directory_modes_after_open()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let directories = [root.join("entries"), root.join("entries/user")];
    for directory in &directories {
        fs::set_permissions(directory, fs::Permissions::from_mode(0o755))?;
    }
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft(
        "Mutable admission permissions.",
        "mutable admission permissions",
        "Established.",
    );

    stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;

    for directory in directories {
        assert_eq!(private_mode(&directory)?, 0o700);
    }
    Ok(())
}

#[test]
fn read_only_audit_refuses_nested_mode_drift_after_open_without_repair()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project = project_scope(fixture.path())?;
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = draft_yaml(
        "project",
        "Read-only audit permissions.",
        "read-only audit permissions",
        "Established.",
        "user-decision",
        "decision:read-only-audit-permissions",
    );
    stored_id(store.admit(
        &resolved(&draft, &context)?,
        Some(&project),
        &timestamp,
        &context,
    ))?;
    let audit = Store::open_read_only(&memory_root(&root)?)?.ok_or("missing fixture value")?;
    let scope = root.join(format!("entries/project/{}", project.key().as_str()));
    fs::set_permissions(&scope, fs::Permissions::from_mode(0o755))?;

    let error = audit.list().err().ok_or("expected operation failure")?;

    assert_eq!(error.code(), "store_permissions_unavailable");
    assert_eq!(private_mode(&scope)?, 0o755);
    Ok(())
}

#[test]
fn retrieval_refuses_nested_mode_drift_after_open_without_repair()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let runner = FakeProcessRunner::default();
    let context = SourceContext::new(fixture.path(), &runner, &runner);
    let timestamp = parse_utc_timestamp("2026-08-28T12:00:00Z")?;
    let draft = user_draft(
        "Retrieval permissions.",
        "retrieval permissions",
        "Established.",
    );
    stored_id(store.admit(&resolved(&draft, &context)?, None, &timestamp, &context))?;
    let retrieval =
        Store::open_for_retrieval(&memory_root(&root)?)?.ok_or("missing fixture value")?;
    let project = project_scope(fixture.path())?;
    let index = Index::load_or_rebuild(&retrieval)?.index;
    let selection = search(
        &index,
        SearchRequest {
            query: "retrieval permissions",
            project_key: project.key(),
            include_user: true,
            limit: 5,
        },
    );
    let scope = root.join("entries/user");
    fs::set_permissions(&scope, fs::Permissions::from_mode(0o755))?;
    let clock = PermissionClock(timestamp);
    let resolver = NeverResolver;

    let report = retrieve(
        RetrievalRequest::new(&selection, project.key(), true),
        &RetrievalContext::new(
            &retrieval,
            &clock,
            &resolver,
            OracleEnvironment::new("macos", "aarch64"),
        ),
    );

    assert_eq!(
        report.omitted.first().ok_or("missing omission")?.code,
        "store_permissions_unavailable"
    );
    assert_eq!(private_mode(&scope)?, 0o755);
    Ok(())
}

fn project_scope(
    directory: &Path,
) -> Result<agent_memory::ProjectScope, Box<dyn std::error::Error + Send + Sync>> {
    let common = directory.join("common.git");
    fs::create_dir_all(&common)?;
    let runner = FakeProcessRunner::with_responses([FakeResponse::success(format!(
        "{}\n",
        common.display()
    ))]);
    Ok(resolve_project(directory, &runner)?)
}

struct PermissionClock(UtcTimestamp);

impl Clock for PermissionClock {
    fn now(&self) -> UtcTimestamp {
        self.0.clone()
    }
}

struct NeverResolver;

impl SourceResolver for NeverResolver {
    fn resolve(&self, _: &agent_memory::EntrySource) -> SourceResolution {
        SourceResolution::Unavailable
    }
}
