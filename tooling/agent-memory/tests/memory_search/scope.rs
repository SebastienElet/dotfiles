use super::support::{admit_project, admit_user, memory_root, project_scope};
use agent_memory::{Index, SearchRequest, SearchSelection, Store, search};
use std::fs;

#[test]
fn isolates_the_requested_project_and_optionally_includes_user_scope()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project_a = project_scope(fixture.path(), "project-a.git")?;
    let project_b = project_scope(fixture.path(), "project-b.git")?;
    let selected_project = admit_project(
        &store,
        fixture.path(),
        &project_a,
        "Agent project A.",
        &["agent"],
        "Established.",
    )?;
    admit_project(
        &store,
        fixture.path(),
        &project_b,
        "Agent project B.",
        &["agent"],
        "Established.",
    )?;
    let selected_user = admit_user(
        &store,
        fixture.path(),
        "Agent user.",
        &["agent"],
        "Established.",
    )?;
    let index = Index::load_or_rebuild(&store)?.index;
    let with_user = search(
        &index,
        SearchRequest {
            query: "agent",
            project_key: project_a.key(),
            include_user: true,
            limit: 5,
        },
    );
    let project_only = search(
        &index,
        SearchRequest {
            query: "agent",
            project_key: project_a.key(),
            include_user: false,
            limit: 5,
        },
    );

    let mut expected = vec![selected_project.clone(), selected_user];
    expected.sort();
    assert_eq!(ids(&with_user), expected);
    assert_eq!(ids(&project_only), vec![selected_project]);
    Ok(())
}

#[test]
fn limits_results_and_reports_only_matches_omitted_by_the_limit()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project = project_scope(fixture.path(), "project-a.git")?;
    for number in 0..6 {
        admit_project(
            &store,
            fixture.path(),
            &project,
            &format!("Agent statement {number}."),
            &["agent"],
            "Established.",
        )?;
    }
    let index = Index::load_or_rebuild(&store)?.index;
    let selection = search(
        &index,
        SearchRequest {
            query: "agent",
            project_key: project.key(),
            include_user: false,
            limit: 5,
        },
    );

    assert_eq!(selection.selected.len(), 5);
    assert_eq!(selection.omitted_by_limit, 1);
    assert!(selection.diagnostics.is_empty());
    Ok(())
}

#[test]
fn clamps_requested_limits_above_five_and_reports_the_remaining_matches()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project = project_scope(fixture.path(), "project-a.git")?;
    for number in 0..6 {
        admit_project(
            &store,
            fixture.path(),
            &project,
            &format!("Bounded agent statement {number}."),
            &["bounded agent"],
            "Established.",
        )?;
    }
    let index = Index::load_or_rebuild(&store)?.index;

    let selection = search(
        &index,
        SearchRequest {
            query: "bounded agent",
            project_key: project.key(),
            include_user: false,
            limit: 6,
        },
    );

    assert_eq!(selection.selected.len(), 5);
    assert_eq!(selection.omitted_by_limit, 1);
    Ok(())
}

#[test]
fn returns_only_diagnostics_visible_to_the_requested_scopes()
-> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let fixture = tempfile::tempdir()?;
    let root = fixture.path().join("agent-memory");
    let store = Store::open(&memory_root(&root)?)?;
    let project_a = project_scope(fixture.path(), "project-a.git")?;
    let project_b = project_scope(fixture.path(), "project-b.git")?;
    let selected_entry_id = admit_project(
        &store,
        fixture.path(),
        &project_a,
        "Private diagnostic A.",
        &["private diagnostic"],
        "Established.",
    )?;
    let other_entry_id = admit_project(
        &store,
        fixture.path(),
        &project_b,
        "Private diagnostic B.",
        &["private diagnostic"],
        "Established.",
    )?;
    let user_id = admit_user(
        &store,
        fixture.path(),
        "Private diagnostic user.",
        &["private diagnostic"],
        "Established.",
    )?;
    for id in [&selected_entry_id, &other_entry_id, &user_id] {
        fs::write(find_yaml(&root, id)?, b"not: [valid")?;
    }
    let index = Index::load_or_rebuild(&store)?.index;

    let project_only = search(
        &index,
        SearchRequest {
            query: "unmatched",
            project_key: project_a.key(),
            include_user: false,
            limit: 5,
        },
    );
    let with_user = search(
        &index,
        SearchRequest {
            query: "unmatched",
            project_key: project_a.key(),
            include_user: true,
            limit: 5,
        },
    );

    assert_eq!(
        diagnostic_ids(&project_only),
        vec![selected_entry_id.clone()]
    );
    let mut expected = vec![selected_entry_id, user_id];
    expected.sort();
    assert_eq!(diagnostic_ids(&with_user), expected);
    assert!(!diagnostic_ids(&with_user).contains(&other_entry_id));
    Ok(())
}

fn ids(selection: &SearchSelection) -> Vec<String> {
    selection
        .selected
        .iter()
        .map(|entry| entry.entry_id.clone())
        .collect()
}

fn diagnostic_ids(selection: &SearchSelection) -> Vec<String> {
    let mut ids = selection
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.entry_id.clone())
        .collect::<Vec<_>>();
    ids.sort();
    ids
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
